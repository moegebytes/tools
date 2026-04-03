use std::io::Cursor;
use std::path::Path;

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::utils::fs::{create_dir, read_file, walk_dir, write_file};
use crate::utils::io::*;
use crate::utils::text::{cmp_sjis_case_insensitive, decode_sjis_lossy, encode_sjis_lossy};

const HEADER_SIZE: usize = 8;
const ENTRY_SIZE: usize = 12;

struct ArchiveEntry {
  name: String,
  data_offset: usize,
  data_size: usize,
}

fn parse_entries(data: &[u8]) -> Result<Vec<ArchiveEntry>> {
  if data.len() < HEADER_SIZE {
    bail!("file too small");
  }

  let mut cur = Cursor::new(data);
  let file_count = read_u32_le(&mut cur)? as usize;
  let names_list_size = read_u32_le(&mut cur)? as usize;

  let names_list_start = HEADER_SIZE + file_count * ENTRY_SIZE;
  let names_list_end = names_list_start + names_list_size;

  if data.len() < names_list_end {
    bail!("file truncated: list of names extends past end of file");
  }

  let names_list = &data[names_list_start..names_list_end];
  let mut entries = Vec::with_capacity(file_count);

  for i in 0..file_count {
    let name_offset = read_u32_le(&mut cur)? as usize;
    let data_offset = read_u32_le(&mut cur)? as usize;
    let data_size = read_u32_le(&mut cur)? as usize;

    if name_offset >= names_list.len() {
      bail!("entry {}: name offset {} out of bounds", i, name_offset);
    }
    let name_end = names_list[name_offset..]
      .iter()
      .position(|&b| b == 0)
      .unwrap_or(names_list.len() - name_offset)
      + name_offset;
    let name_bytes = &names_list[name_offset..name_end];

    let (decoded, had_errors) = decode_sjis_lossy(name_bytes);
    if had_errors {
      eprintln!("warning: entry {}: lossy sjis decode for filename", i);
    }

    if data_offset + data_size > data.len() {
      bail!(
        "entry {}: data extends past end of file (offset={}, size={})",
        i,
        data_offset,
        data_size
      );
    }

    entries.push(ArchiveEntry {
      name: decoded.to_string(),
      data_offset,
      data_size,
    });
  }

  Ok(entries)
}

pub fn ls(archive: &Path) -> Result<()> {
  let data = read_file(archive)?;
  let entries = parse_entries(&data)?;

  for entry in &entries {
    println!("{}\t\t{}", entry.data_size, entry.name.bold());
  }

  eprintln!("{} files", entries.len());
  Ok(())
}

pub fn get(archive: &Path, name: &str, output: &Path) -> Result<()> {
  let data = read_file(archive)?;
  let entries = parse_entries(&data)?;

  let name_upper = name.to_ascii_uppercase();
  let entry = entries
    .iter()
    .find(|e| e.name.to_ascii_uppercase() == name_upper)
    .with_context(|| format!("file {} not found in archive", name))?;

  let file_data = &data[entry.data_offset..entry.data_offset + entry.data_size];
  write_file(output, file_data)?;

  eprintln!("unpacked '{}' ({} bytes)", entry.name, entry.data_size);
  Ok(())
}

pub fn replace(archive: &Path, name: &str, file: &Path) -> Result<()> {
  let data = read_file(archive)?;
  let entries = parse_entries(&data)?;

  let name_upper = name.to_ascii_uppercase();
  let index = entries
    .iter()
    .position(|e| e.name.to_ascii_uppercase() == name_upper)
    .with_context(|| format!("file {} not found in archive", name))?;

  let new_content = read_file(file)?;

  let file_count = entries.len();
  let header_and_names_end = entries.iter().map(|e| e.data_offset).min().unwrap_or(data.len());
  let header = &data[..header_and_names_end];

  let old_size = entries[index].data_size;
  let size_diff = new_content.len() as i64 - old_size as i64;

  let mut out = Vec::with_capacity((data.len() as i64 + size_diff) as usize);
  out.extend_from_slice(header);

  for (i, entry) in entries.iter().enumerate() {
    if i == index {
      out.extend_from_slice(&new_content);
    } else {
      out.extend_from_slice(&data[entry.data_offset..entry.data_offset + entry.data_size]);
    }
  }

  let mut current_offset = header_and_names_end;
  for (i, entry) in entries.iter().enumerate().take(file_count) {
    let table_off = HEADER_SIZE + i * ENTRY_SIZE;
    let size = if i == index { new_content.len() } else { entry.data_size };
    out[table_off + 4..table_off + 8].copy_from_slice(&(current_offset as u32).to_le_bytes());
    out[table_off + 8..table_off + 12].copy_from_slice(&(size as u32).to_le_bytes());
    current_offset += size;
  }

  write_file(archive, &out)?;

  eprintln!(
    "replaced '{}' ({} -> {} bytes)",
    entries[index].name,
    old_size,
    new_content.len()
  );
  Ok(())
}

pub fn unpack(archive: &Path, output_folder: &Path) -> Result<()> {
  let data = read_file(archive)?;
  let entries = parse_entries(&data)?;

  create_dir(output_folder)?;

  for entry in &entries {
    let output_path = output_folder.join(&entry.name);
    let file_data = &data[entry.data_offset..entry.data_offset + entry.data_size];
    write_file(&output_path, file_data)?;
    eprintln!("unpacked '{}' ({} bytes)", entry.name, entry.data_size);
  }

  eprintln!("done: {} files unpacked", entries.len());
  Ok(())
}

pub fn pack(input_folder: &Path, output: &Path) -> Result<()> {
  let files = walk_dir(input_folder)?;

  let mut entries: Vec<(String, Vec<u8>, &Path)> = Vec::with_capacity(files.len());
  for path in &files {
    let file_name = path.file_name().unwrap().to_string_lossy().to_string();

    if let Some(prev) = entries.last() {
      if prev.0.eq_ignore_ascii_case(&file_name) {
        bail!(
          "duplicate filename '{}': '{}' and '{}'",
          file_name,
          prev.2.display(),
          path.display()
        );
      }
    }

    let (sjis_bytes, had_errors) = encode_sjis_lossy(&file_name);
    if had_errors {
      eprintln!("warning: lossy sjis encode for filename '{}'", file_name);
    }

    entries.push((file_name, sjis_bytes, path));
  }

  let file_count = entries.len();
  let names_list_size: usize = entries.iter().map(|(_, sjis, _)| sjis.len() + 1).sum();

  let mut names_list = Vec::with_capacity(names_list_size);
  let mut name_offsets = Vec::with_capacity(file_count);
  for (_, sjis, _) in &entries {
    name_offsets.push(names_list.len() as u32);
    names_list.extend_from_slice(sjis);
    names_list.push(0); // null terminator
  }

  let names_list_start = HEADER_SIZE + file_count * ENTRY_SIZE;
  let file_data_start = names_list_start + names_list.len();

  let mut file_data_buf = Vec::new();
  let mut file_offsets = Vec::with_capacity(file_count);
  let mut file_sizes = Vec::with_capacity(file_count);

  for (file_name, _, path) in &entries {
    let content = read_file(path)?;

    let offset = file_data_start + file_data_buf.len();
    let size = content.len();
    file_offsets.push(offset as u32);
    file_sizes.push(size as u32);
    file_data_buf.extend_from_slice(&content);

    eprintln!("packing '{}' ({} bytes)", file_name, size);
  }

  let total_size = file_data_start + file_data_buf.len();
  let mut out = Vec::with_capacity(total_size);

  out.extend_from_slice(&(file_count as u32).to_le_bytes());
  out.extend_from_slice(&(names_list_size as u32).to_le_bytes());

  for i in 0..file_count {
    out.extend_from_slice(&name_offsets[i].to_le_bytes());
    out.extend_from_slice(&file_offsets[i].to_le_bytes());
    out.extend_from_slice(&file_sizes[i].to_le_bytes());
  }

  out.extend_from_slice(&names_list);
  out.extend_from_slice(&file_data_buf);

  write_file(output, &out)?;

  eprintln!("done: {} files packed into '{}'", file_count, output.display());
  Ok(())
}

pub fn validate(archive: &Path) -> Result<()> {
  let data = read_file(archive)?;
  let entries = parse_entries(&data)?;

  let mut warnings = 0u32;

  let mut regions: Vec<(usize, usize, &str)> = entries
    .iter()
    .map(|e| (e.data_offset, e.data_offset + e.data_size, e.name.as_str()))
    .collect();
  regions.sort_by_key(|r| (r.0, r.1));

  for pair in regions.windows(2) {
    let (_, end_a, name_a) = pair[0];
    let (start_b, _, name_b) = pair[1];
    if end_a > start_b {
      eprintln!("warning: overlapping data: '{}' and '{}'", name_a, name_b);
      warnings += 1;
    }
  }

  let sjis_names: Vec<Vec<u8>> = entries.iter().map(|e| encode_sjis_lossy(&e.name).0).collect();

  for pair in sjis_names.windows(2) {
    if cmp_sjis_case_insensitive(&pair[0], &pair[1]).is_gt() {
      let (a, _) = decode_sjis_lossy(&pair[0]);
      let (b, _) = decode_sjis_lossy(&pair[1]);
      eprintln!("warning: sort order violation: '{}' should come after '{}'", a, b);
      warnings += 1;
    }
  }

  if warnings > 0 {
    bail!("archive validation failed with {} warning(s)", warnings);
  }

  eprintln!("ok: {} entries, all checks passed", entries.len());
  Ok(())
}
