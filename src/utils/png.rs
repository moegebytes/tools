use std::fs;
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result};

pub fn save_png(output: &Path, width: u32, height: u32, color_type: png::ColorType, data: &[u8]) -> Result<()> {
  let file = fs::File::create(output).with_context(|| format!("failed to create '{}'", output.display()))?;
  let w = BufWriter::new(file);
  let mut encoder = png::Encoder::new(w, width, height);
  encoder.set_color(color_type);
  encoder.set_depth(png::BitDepth::Eight);
  encoder.set_compression(png::Compression::Fast);
  let mut writer = encoder
    .write_header()
    .with_context(|| format!("failed to write png header to '{}'", output.display()))?;
  writer
    .write_image_data(data)
    .with_context(|| format!("failed to write png data to '{}'", output.display()))?;
  Ok(())
}

pub fn load_png(input: &Path) -> Result<(png::ColorType, u16, u16, Vec<u8>)> {
  let file = fs::File::open(input).with_context(|| format!("failed to open '{}'", input.display()))?;
  let decoder = png::Decoder::new(std::io::BufReader::new(file));
  let mut reader = decoder
    .read_info()
    .with_context(|| format!("failed to read png header from '{}'", input.display()))?;
  let mut buf = vec![0u8; reader.output_buffer_size().context("png output buffer size unknown")?];
  let info = reader
    .next_frame(&mut buf)
    .with_context(|| format!("failed to decode png from '{}'", input.display()))?;
  buf.truncate(info.buffer_size());
  Ok((info.color_type, info.width as u16, info.height as u16, buf))
}
