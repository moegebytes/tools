use std::io::{Cursor, Read};

use anyhow::Result;

pub fn read_u8(cur: &mut Cursor<&[u8]>) -> Result<u8> {
  let mut buf = [0u8; 1];
  cur.read_exact(&mut buf)?;
  Ok(buf[0])
}

pub fn read_i8(cur: &mut Cursor<&[u8]>) -> Result<i8> {
  Ok(read_u8(cur)? as i8)
}

pub fn read_u16_le(cur: &mut Cursor<&[u8]>) -> Result<u16> {
  let mut buf = [0u8; 2];
  cur.read_exact(&mut buf)?;
  Ok(u16::from_le_bytes(buf))
}

pub fn read_u16_be(cur: &mut Cursor<&[u8]>) -> Result<u16> {
  let mut buf = [0u8; 2];
  cur.read_exact(&mut buf)?;
  Ok(u16::from_be_bytes(buf))
}

pub fn read_i16_le(cur: &mut Cursor<&[u8]>) -> Result<i16> {
  let mut buf = [0u8; 2];
  cur.read_exact(&mut buf)?;
  Ok(i16::from_le_bytes(buf))
}

pub fn read_i16_be(cur: &mut Cursor<&[u8]>) -> Result<i16> {
  let mut buf = [0u8; 2];
  cur.read_exact(&mut buf)?;
  Ok(i16::from_be_bytes(buf))
}

pub fn read_u32_le(cur: &mut Cursor<&[u8]>) -> Result<u32> {
  let mut buf = [0u8; 4];
  cur.read_exact(&mut buf)?;
  Ok(u32::from_le_bytes(buf))
}

pub fn read_u32_be(cur: &mut Cursor<&[u8]>) -> Result<u32> {
  let mut buf = [0u8; 4];
  cur.read_exact(&mut buf)?;
  Ok(u32::from_be_bytes(buf))
}

pub fn read_i32_le(cur: &mut Cursor<&[u8]>) -> Result<i32> {
  let mut buf = [0u8; 4];
  cur.read_exact(&mut buf)?;
  Ok(i32::from_le_bytes(buf))
}

pub fn read_i32_be(cur: &mut Cursor<&[u8]>) -> Result<i32> {
  let mut buf = [0u8; 4];
  cur.read_exact(&mut buf)?;
  Ok(i32::from_be_bytes(buf))
}

pub fn read_f32_le(cur: &mut Cursor<&[u8]>) -> Result<f32> {
  let mut buf = [0u8; 4];
  cur.read_exact(&mut buf)?;
  Ok(f32::from_le_bytes(buf))
}

pub fn read_f32_be(cur: &mut Cursor<&[u8]>) -> Result<f32> {
  let mut buf = [0u8; 4];
  cur.read_exact(&mut buf)?;
  Ok(f32::from_be_bytes(buf))
}

pub fn read_bytes(cur: &mut Cursor<&[u8]>, len: usize) -> Result<Vec<u8>> {
  let mut buf = vec![0u8; len];
  cur.read_exact(&mut buf)?;
  Ok(buf)
}
