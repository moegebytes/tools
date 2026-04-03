use std::io::{Cursor, Read, Write};

use anyhow::{bail, Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use super::io::*;

const HZC1_MAGIC: &[u8; 4] = b"hzc1";

pub fn read_prefix(data: &[u8]) -> Result<&[u8]> {
  if data.len() < 4 || &data[0..4] != HZC1_MAGIC {
    bail!("not an hzc1 file");
  }
  if data.len() < 12 {
    bail!("hzc1 header truncated");
  }

  let mut cur = Cursor::new(data);
  cur.set_position(8);
  let prefix_size = read_u32_le(&mut cur)? as usize;
  if data.len() < 12 + prefix_size {
    bail!("file truncated: prefix extends past end");
  }

  Ok(&data[12..12 + prefix_size])
}

pub fn decompress(data: &[u8]) -> Result<(&[u8], Vec<u8>)> {
  if data.len() < 4 || &data[0..4] != HZC1_MAGIC {
    bail!("not an hzc1 file");
  }

  if data.len() < 12 {
    bail!("hzc1 header truncated");
  }

  let mut cur = Cursor::new(data);
  cur.set_position(4);
  let uncompressed_size = read_u32_le(&mut cur)? as usize;
  let prefix_size = read_u32_le(&mut cur)? as usize;

  if data.len() < 12 + prefix_size {
    bail!("file truncated: prefix extends past end");
  }

  let prefix = &data[12..12 + prefix_size];
  let compressed = &data[12 + prefix_size..];

  let mut decompressed = Vec::with_capacity(uncompressed_size);
  let mut decoder = ZlibDecoder::new(compressed);
  decoder
    .read_to_end(&mut decompressed)
    .context("zlib decompression failed")?;

  if decompressed.len() != uncompressed_size {
    bail!(
      "decompressed size mismatch: expected {}, got {}",
      uncompressed_size,
      decompressed.len()
    );
  }

  Ok((prefix, decompressed))
}

pub fn compress(prefix: &[u8], pixel_data: &[u8]) -> Result<Vec<u8>> {
  let mut compressed = Vec::new();
  {
    let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
    encoder.write_all(pixel_data).context("zlib compression failed")?;
    encoder.finish().context("zlib compression finish failed")?;
  }

  let uncompressed_size = pixel_data.len() as u32;
  let prefix_size = prefix.len() as u32;
  let mut out = Vec::with_capacity(12 + prefix.len() + compressed.len());
  out.extend_from_slice(HZC1_MAGIC);
  out.extend_from_slice(&uncompressed_size.to_le_bytes());
  out.extend_from_slice(&prefix_size.to_le_bytes());
  out.extend_from_slice(prefix);
  out.extend_from_slice(&compressed);

  Ok(out)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bad_magic() {
    let result = decompress(b"deadbeef");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not an hzc1 file"));
  }

  #[test]
  fn truncated_header() {
    let result = decompress(b"hzc1\x00\x00");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("truncated"));
  }

  #[test]
  fn compress_decompress_roundtrip() {
    let prefix = b"prefix";
    let pixels = b"some pixel data that should survive the roundtrip";

    let compressed = compress(prefix, pixels).unwrap();
    let (got_prefix, got_pixels) = decompress(&compressed).unwrap();

    assert_eq!(got_prefix, prefix);
    assert_eq!(got_pixels, pixels);
  }

  #[test]
  fn read_prefix_extracts_prefix() {
    let prefix = b"prefix";
    let pixels = b"some pixel data";
    let compressed = compress(prefix, pixels).unwrap();

    let got = read_prefix(&compressed).unwrap();
    assert_eq!(got, prefix);
  }

  #[test]
  fn prefix_truncated() {
    let mut data = Vec::new();
    data.extend_from_slice(b"hzc1");
    data.extend_from_slice(&0u32.to_le_bytes()); // uncompressed_size
    data.extend_from_slice(&100u32.to_le_bytes()); // prefix_size = 100, but no data follows

    let result = decompress(&data);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("prefix extends past end"));
  }
}
