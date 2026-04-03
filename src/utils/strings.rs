use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::fs::{read_file_to_string, write_file};

pub fn load(path: &Path) -> Result<Vec<String>> {
  let mut strings = Vec::new();
  load_inner(path, &mut strings, &mut Vec::new())?;
  Ok(strings)
}

fn load_inner(path: &Path, strings: &mut Vec<String>, seen: &mut Vec<PathBuf>) -> Result<()> {
  let canonical = path
    .canonicalize()
    .with_context(|| format!("resolving path '{}'", path.display()))?;
  if seen.contains(&canonical) {
    bail!(
      "circular include detected: '{}' is already in the include stack",
      path.display()
    );
  }
  seen.push(canonical);

  let text = read_file_to_string(path)?;
  let parent = path.parent().unwrap_or(Path::new("."));

  for line in text.lines() {
    if line.is_empty() || line.starts_with(";") {
      continue;
    }
    if let Some(rest) = line.strip_prefix("#reference <") {
      if rest.strip_suffix('>').is_some() {
        continue;
      }
    }
    if let Some(rest) = line.strip_prefix("#include <") {
      if let Some(filename) = rest.strip_suffix('>') {
        let include_path = parent.join(filename);
        load_inner(&include_path, strings, seen).with_context(|| format!("processing include <{}>", filename))?;
        continue;
      }
    }
    if let Some(rest) = line.strip_prefix("#emit") {
      let keyword = rest.trim();
      match keyword {
        "empty" => strings.push(String::new()),
        _ => bail!("unknown #emit keyword: '{}'", keyword),
      }
      continue;
    }
    strings.push(line.to_string());
  }

  seen.pop();
  Ok(())
}

pub fn save(path: &Path, strings: &[String]) -> Result<()> {
  let mut buf = String::new();
  for s in strings {
    if s.is_empty() {
      buf.push_str("#emit empty\n");
    } else {
      buf.push_str(s);
      buf.push('\n');
    }
  }
  write_file(path, buf.as_bytes())
}
