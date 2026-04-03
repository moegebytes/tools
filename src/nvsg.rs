use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{bail, Context, Result};
use colored::Colorize;
use rayon::prelude::*;

use crate::utils::bitmap::*;
use crate::utils::fs::{create_dir, read_file, walk_dir, write_file};
use crate::utils::hzc1;
use crate::utils::io::*;
use crate::utils::png::{load_png, save_png};

const NVSG_MAGIC: &[u8; 4] = b"NVSG";
const NVSG_HEADER_SIZE: usize = 32;
const NVSG_MAX_DIMENSION: u16 = 4096;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ImageType {
  Bgr = 0,
  Bgra = 1,
  Parts = 2,
  Mask = 3,
  Gaiji = 4,
}

impl TryFrom<u16> for ImageType {
  type Error = anyhow::Error;
  fn try_from(v: u16) -> Result<Self> {
    match v {
      0 => Ok(Self::Bgr),
      1 => Ok(Self::Bgra),
      2 => Ok(Self::Parts),
      3 => Ok(Self::Mask),
      4 => Ok(Self::Gaiji),
      _ => bail!("unknown image type: {}", v),
    }
  }
}

pub struct EncodeOptions {
  pub image_type: Option<ImageType>,
  pub offset_x: u16,
  pub offset_y: u16,
  pub anchor_x: u16,
  pub anchor_y: u16,
  pub parts_count: u16,
}

struct NvsgHeader {
  version: u16,
  image_type: ImageType,
  width: i16,
  height: i16,
  offset_x: i16,
  offset_y: i16,
  anchor_x: i16,
  anchor_y: i16,
  parts_count: i16,
}

fn parse_header(data: &[u8]) -> Result<NvsgHeader> {
  if data.len() < 4 || &data[0..4] != NVSG_MAGIC {
    bail!("not an nvsg file");
  }

  if data.len() < NVSG_HEADER_SIZE {
    bail!("nvsg header truncated");
  }

  let mut cur = Cursor::new(data);
  cur.set_position(4);

  let version = read_u16_be(&mut cur)?;
  if version != 1 {
    bail!("unsupported version: {}", version);
  }
  let image_type = ImageType::try_from(read_u16_le(&mut cur)?)?;
  let width = read_i16_le(&mut cur)?;
  let height = read_i16_le(&mut cur)?;
  let offset_x = read_i16_le(&mut cur)?;
  let offset_y = read_i16_le(&mut cur)?;
  let anchor_x = read_i16_le(&mut cur)?;
  let anchor_y = read_i16_le(&mut cur)?;
  let parts_count = read_i16_le(&mut cur)?;

  if width <= 0 || height <= 0 {
    bail!("invalid dimensions: {}x{}", width, height);
  }
  if parts_count < 0 {
    bail!("invalid parts count: {}", parts_count);
  }

  Ok(NvsgHeader {
    version,
    image_type,
    width,
    height,
    offset_x,
    offset_y,
    anchor_x,
    anchor_y,
    parts_count,
  })
}

fn serialize_header(h: &NvsgHeader) -> Vec<u8> {
  let mut buf = Vec::with_capacity(NVSG_HEADER_SIZE);
  buf.extend_from_slice(NVSG_MAGIC);
  buf.extend_from_slice(&h.version.to_be_bytes());
  buf.extend_from_slice(&(h.image_type as u16).to_le_bytes());
  buf.extend_from_slice(&h.width.to_le_bytes());
  buf.extend_from_slice(&h.height.to_le_bytes());
  buf.extend_from_slice(&h.offset_x.to_le_bytes());
  buf.extend_from_slice(&h.offset_y.to_le_bytes());
  buf.extend_from_slice(&h.anchor_x.to_le_bytes());
  buf.extend_from_slice(&h.anchor_y.to_le_bytes());
  buf.extend_from_slice(&h.parts_count.to_le_bytes());
  buf.resize(NVSG_HEADER_SIZE, 0);
  buf
}

fn type_name(t: ImageType) -> &'static str {
  match t {
    ImageType::Bgr => "BGR",
    ImageType::Bgra => "BGRA",
    ImageType::Parts => "Parts",
    ImageType::Mask => "Mask",
    ImageType::Gaiji => "Gaiji",
  }
}

pub fn info(input: &Path) -> Result<()> {
  let data = read_file(input)?;
  let prefix = hzc1::read_prefix(&data)?;
  let header = parse_header(prefix)?;

  println!(
    "{} {} ({})",
    "Type:".bold(),
    type_name(header.image_type),
    header.image_type as u16
  );
  println!("{} {}x{}", "Size:".bold(), header.width, header.height);
  println!("{} ({}, {})", "Offset:".bold(), header.offset_x, header.offset_y);
  println!("{} ({}, {})", "Anchor:".bold(), header.anchor_x, header.anchor_y);
  println!("{} {}", "Parts:".bold(), header.parts_count);

  let compressed_size = data.len() - 12 - prefix.len();
  let pixel_bpp: u32 = match header.image_type {
    ImageType::Bgr => 3,
    ImageType::Bgra | ImageType::Parts => 4,
    ImageType::Mask | ImageType::Gaiji => 1,
  };
  let total_h = if header.image_type == ImageType::Parts {
    (header.height as u32)
      .checked_mul(header.parts_count as u32)
      .with_context(|| "total height overflow")?
  } else {
    header.height as u32
  };
  let uncompressed_size = (header.width as u32)
    .checked_mul(total_h)
    .and_then(|v| v.checked_mul(pixel_bpp))
    .with_context(|| "pixel data size overflow")?;

  println!(
    "{} {} -> {} bytes ({:.1}%)",
    "Compression:".bold(),
    uncompressed_size,
    compressed_size,
    compressed_size as f64 / uncompressed_size as f64 * 100.0,
  );

  Ok(())
}

fn decode_single(input: &Path, output: &Path) -> Result<()> {
  let data = read_file(input)?;

  let (prefix, mut pixel_data) = hzc1::decompress(&data)?;
  let header = parse_header(prefix)?;

  if header.image_type != ImageType::Parts && header.parts_count > 0 {
    bail!(
      "got {} parts but image type {} can't have any",
      header.parts_count,
      header.image_type as u16
    );
  }

  let w = header.width as u32;
  let h = header.height as u32;
  let parts_count = header.parts_count;

  match header.image_type {
    ImageType::Bgr => {
      let expected = w
        .checked_mul(h)
        .and_then(|v| v.checked_mul(3))
        .with_context(|| "pixel data size overflow")? as usize;
      if pixel_data.len() < expected {
        bail!("pixel data too small: expected {}, got {}", expected, pixel_data.len());
      }
      swap_bgr_rgb(&mut pixel_data, 3);
      save_png(output, w, h, png::ColorType::Rgb, &pixel_data)?;
    }
    ImageType::Bgra => {
      let expected = w
        .checked_mul(h)
        .and_then(|v| v.checked_mul(4))
        .with_context(|| "pixel data size overflow")? as usize;
      if pixel_data.len() < expected {
        bail!("pixel data too small: expected {}, got {}", expected, pixel_data.len());
      }
      swap_bgr_rgb(&mut pixel_data, 4);
      save_png(output, w, h, png::ColorType::Rgba, &pixel_data)?;
    }
    ImageType::Parts => {
      let total_h = h
        .checked_mul(parts_count as u32)
        .with_context(|| "total height overflow")?;
      let expected = w
        .checked_mul(total_h)
        .and_then(|v| v.checked_mul(4))
        .with_context(|| "pixel data size overflow")? as usize;
      if pixel_data.len() < expected {
        bail!("pixel data too small: expected {}, got {}", expected, pixel_data.len());
      }
      swap_bgr_rgb(&mut pixel_data, 4);
      save_png(output, w, total_h, png::ColorType::Rgba, &pixel_data)?;
    }
    ImageType::Mask | ImageType::Gaiji => {
      let expected = w.checked_mul(h).with_context(|| "pixel data size overflow")? as usize;
      if pixel_data.len() < expected {
        bail!("pixel data too small: expected {}, got {}", expected, pixel_data.len());
      }
      let rgba = rgba_from_mask(&pixel_data);
      save_png(output, w, h, png::ColorType::Rgba, &rgba)?;
    }
  }

  eprintln!(
    "decoded: type={} size={}x{} offset=({},{}) anchor=({},{}) parts={} <- '{}'",
    header.image_type as u16,
    header.width,
    header.height,
    header.offset_x,
    header.offset_y,
    header.anchor_x,
    header.anchor_y,
    header.parts_count,
    input.display()
  );

  Ok(())
}

pub fn decode(input: &Path, output: &Path) -> Result<()> {
  if input.is_dir() {
    create_dir(output)?;

    let files = walk_dir(input)?;

    for file in &files {
      let rel = file.strip_prefix(input).unwrap();
      let out_path = output.join(rel).with_extension("png");
      if let Some(parent) = out_path.parent() {
        create_dir(parent)?;
      }
    }

    let errors = AtomicU32::new(0);

    files.par_iter().for_each(|file| {
      let rel = file.strip_prefix(input).unwrap();
      let out_path = output.join(rel).with_extension("png");

      if let Err(e) = decode_single(file, &out_path) {
        errors.fetch_add(1, Ordering::Relaxed);
        eprintln!("warning: {}: {}", file.display(), e);
      }
    });

    let error_count = errors.load(Ordering::Relaxed);
    if error_count > 0 {
      bail!("{} file(s) failed to decode", error_count);
    }
  } else {
    decode_single(input, output)?;
  }

  Ok(())
}

fn determine_type_and_pixels(
  png_color: png::ColorType,
  img_width: u16,
  img_height: u16,
  data: Vec<u8>,
  opts: &EncodeOptions,
) -> Result<(ImageType, Vec<u8>, u16, u16)> {
  let to_rgb = |data: Vec<u8>| -> Vec<u8> {
    match png_color {
      png::ColorType::Rgb => data,
      png::ColorType::Rgba => rgb_from_rgba(&data),
      png::ColorType::Grayscale => data.iter().flat_map(|&g| [g, g, g]).collect(),
      png::ColorType::GrayscaleAlpha => data.chunks_exact(2).flat_map(|ga| [ga[0], ga[0], ga[0]]).collect(),
      _ => data,
    }
  };
  let to_rgba = |data: Vec<u8>| -> Vec<u8> {
    match png_color {
      png::ColorType::Rgba => data,
      png::ColorType::Rgb => rgba_from_rgb(&data),
      png::ColorType::Grayscale => data.iter().flat_map(|&g| [g, g, g, 0xFF]).collect(),
      png::ColorType::GrayscaleAlpha => data
        .chunks_exact(2)
        .flat_map(|ga| [ga[0], ga[0], ga[0], ga[1]])
        .collect(),
      _ => data,
    }
  };
  let to_mask = |data: Vec<u8>| -> Vec<u8> {
    match png_color {
      png::ColorType::Grayscale => data,
      png::ColorType::GrayscaleAlpha => data.chunks_exact(2).map(|ga| ga[0]).collect(),
      png::ColorType::Rgb => grayscale_from_rgb(&data),
      png::ColorType::Rgba => mask_from_rgba(&data),
      _ => data,
    }
  };

  if let Some(image_type) = opts.image_type {
    let (pixel_data, width, height) = match image_type {
      ImageType::Bgr => (to_rgb(data), img_width, img_height),
      ImageType::Bgra => (to_rgba(data), img_width, img_height),
      ImageType::Parts => {
        if !img_height.is_multiple_of(opts.parts_count) {
          bail!(
            "image height {} is not divisible by {} parts",
            img_height,
            opts.parts_count
          );
        }
        let frame_height = img_height / opts.parts_count;
        (to_rgba(data), img_width, frame_height)
      }
      ImageType::Mask | ImageType::Gaiji => (to_mask(data), img_width, img_height),
    };
    Ok((image_type, pixel_data, width, height))
  } else if opts.parts_count > 1 {
    if !img_height.is_multiple_of(opts.parts_count) {
      bail!(
        "image height {} is not divisible by {} parts",
        img_height,
        opts.parts_count
      );
    }
    let frame_height = img_height / opts.parts_count;
    Ok((ImageType::Parts, to_rgba(data), img_width, frame_height))
  } else {
    match png_color {
      png::ColorType::Rgba | png::ColorType::GrayscaleAlpha => {
        Ok((ImageType::Bgra, to_rgba(data), img_width, img_height))
      }
      png::ColorType::Rgb => Ok((ImageType::Bgr, data, img_width, img_height)),
      png::ColorType::Grayscale => Ok((ImageType::Mask, data, img_width, img_height)),
      _ => Ok((ImageType::Bgra, to_rgba(data), img_width, img_height)),
    }
  }
}

pub fn encode(input: &Path, output: &Path, opts: &EncodeOptions) -> Result<()> {
  let (png_color, img_width, img_height, pixel_data) = load_png(input)?;

  let (image_type, mut pixel_data, width, height) =
    determine_type_and_pixels(png_color, img_width, img_height, pixel_data, opts)?;

  if width == 0 || height == 0 {
    bail!("invalid dimensions: {}x{}", width, height);
  }
  if width > NVSG_MAX_DIMENSION || height > NVSG_MAX_DIMENSION {
    bail!(
      "dimensions {}x{} exceed engine limit of {}x{}",
      width,
      height,
      NVSG_MAX_DIMENSION,
      NVSG_MAX_DIMENSION
    );
  }

  if opts.parts_count > i16::MAX as u16 {
    bail!("parts count {} exceeds engine limit of {}", opts.parts_count, i16::MAX);
  }

  if image_type != ImageType::Parts && opts.parts_count > 0 {
    bail!("image type {} can't have parts", image_type as u16);
  }

  match image_type {
    ImageType::Bgr => swap_bgr_rgb(&mut pixel_data, 3),
    ImageType::Bgra | ImageType::Parts => swap_bgr_rgb(&mut pixel_data, 4),
    ImageType::Mask | ImageType::Gaiji => {}
  }

  let header = NvsgHeader {
    version: 1,
    image_type,
    width: width as i16,
    height: height as i16,
    offset_x: opts.offset_x as i16,
    offset_y: opts.offset_y as i16,
    anchor_x: opts.anchor_x as i16,
    anchor_y: opts.anchor_y as i16,
    parts_count: opts.parts_count as i16,
  };

  let header_bytes = serialize_header(&header);
  let out_data = hzc1::compress(&header_bytes, &pixel_data)?;

  write_file(output, &out_data)?;

  eprintln!(
    "encoded: type={} offset=({},{}) anchor=({},{}) parts={} -> '{}'",
    image_type as u16,
    opts.offset_x,
    opts.offset_y,
    opts.anchor_x,
    opts.anchor_y,
    opts.parts_count,
    output.display()
  );

  Ok(())
}
