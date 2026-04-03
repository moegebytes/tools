use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::text::{cmp_sjis_case_insensitive, encode_sjis_lossy};

pub fn read_file(path: &Path) -> Result<Vec<u8>> {
  fs::read(path).with_context(|| format!("failed to read '{}'", path.display()))
}

pub fn write_file(path: &Path, data: &[u8]) -> Result<()> {
  fs::write(path, data).with_context(|| format!("failed to write '{}'", path.display()))
}

pub fn read_file_to_string(path: &Path) -> Result<String> {
  fs::read_to_string(path).with_context(|| format!("failed to read '{}'", path.display()))
}

pub fn create_dir(path: &Path) -> Result<()> {
  fs::create_dir_all(path).with_context(|| format!("failed to create '{}'", path.display()))
}

pub fn walk_dir(dir: &Path) -> Result<Vec<PathBuf>> {
  let mut files = Vec::new();
  let mut stack = vec![dir.to_path_buf()];
  while let Some(current) = stack.pop() {
    for entry in fs::read_dir(&current).with_context(|| format!("failed to read directory '{}'", current.display()))? {
      let path = entry?.path();
      if path.is_dir() {
        stack.push(path);
      } else {
        files.push(path);
      }
    }
  }
  files.sort_by(|a, b| {
    let a_name = encode_sjis_lossy(&a.file_name().unwrap_or_default().to_string_lossy()).0;
    let b_name = encode_sjis_lossy(&b.file_name().unwrap_or_default().to_string_lossy()).0;
    cmp_sjis_case_insensitive(&a_name, &b_name)
  });
  Ok(files)
}
