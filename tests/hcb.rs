use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

fn fixtures() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

struct DisasmOutput {
  asm: String,
  strings: Vec<String>,
  config: Value,
}

fn disasm_hime() -> DisasmOutput {
  let tmp = tempfile::tempdir().unwrap();
  let out_dir = tmp.path().join("hime");
  hime_tools::hcb::disasm(&fixtures().join("hime.hcb"), &out_dir).unwrap();

  let asm = fs::read_to_string(out_dir.join("hime.asm")).unwrap();
  let strings = hime_tools::utils::strings::load(&out_dir.join("strings.txt")).unwrap();
  let config: Value = serde_yaml::from_str(&fs::read_to_string(out_dir.join("config.yaml")).unwrap()).unwrap();
  DisasmOutput { asm, strings, config }
}

#[test]
fn entry_point_matches_label() {
  let out = disasm_hime();
  let entry = out.config["entry_point"].as_str().unwrap();
  assert_eq!(entry, "sub_0000BA");
  assert!(out.asm.contains("sub_0000BA:"));
}

#[test]
fn call_targets_match_function_labels() {
  let out = disasm_hime();
  for line in out.asm.lines() {
    let line = line.trim();
    if line.starts_with("call LABEL:") {
      let label = line.strip_prefix("call LABEL:").unwrap();
      assert!(
        out.asm.contains(&format!("{}:", label)),
        "call target {} has no matching label",
        label
      );
    }
  }
}

#[test]
fn jump_targets_match_labels() {
  let out = disasm_hime();
  for line in out.asm.lines() {
    let line = line.trim();
    for prefix in &["jz LABEL:", "jmp LABEL:"] {
      if line.starts_with(prefix) {
        let label = line.strip_prefix(prefix).unwrap();
        assert!(
          out.asm.contains(&format!("{}:", label)),
          "{} target {} has no matching label",
          prefix.trim_end_matches(" LABEL:"),
          label
        );
      }
    }
  }
}

#[test]
fn strings_extracted() {
  let out = disasm_hime();
  assert_eq!(out.strings.len(), 2);
  assert_eq!(
    out.strings[0],
    "Thank you very much. May your tomorrow be filled with much more wonderful things than today."
  );
  assert_eq!(out.strings[1], " From all the staff ");
  assert!(out.asm.contains("push_string STRING:1"));
  assert!(out.asm.contains("push_string STRING:2"));
}

#[test]
fn config_metadata() {
  let out = disasm_hime();
  assert_eq!(out.config["game_mode"].as_u64().unwrap(), 2);
  assert_eq!(out.config["game_title"].as_str().unwrap(), "Example Project");

  let syscalls = out.config["syscalls"].as_sequence().unwrap();
  let names: Vec<&str> = syscalls.iter().map(|s| s["name"].as_str().unwrap()).collect();
  assert!(names.contains(&"Debmess"));
  assert!(names.contains(&"ExitMode"));
  assert!(names.contains(&"ThreadNext"));
  assert!(names.contains(&"ThreadExit"));

  let custom_syscalls = out.config["custom_syscalls"].as_sequence().unwrap();
  let custom_names: Vec<&str> = custom_syscalls.iter().map(|s| s["name"].as_str().unwrap()).collect();
  assert!(custom_names.contains(&"SysCustom"));
}

#[test]
fn all_functions_have_labels() {
  let out = disasm_hime();
  let func_count = out.asm.lines().filter(|l| l.starts_with("sub_")).count();
  assert_eq!(func_count, 8, "expected 8 function labels");
}

#[test]
fn roundtrip_disasm_asm() {
  let tmp = tempfile::tempdir().unwrap();
  let disasm_dir = tmp.path().join("hime");
  let reassembled = tmp.path().join("roundtrip.hcb");

  hime_tools::hcb::disasm(&fixtures().join("hime.hcb"), &disasm_dir).unwrap();
  hime_tools::hcb::asm(&disasm_dir, &reassembled).unwrap();

  let original = fs::read(fixtures().join("hime.hcb")).unwrap();
  let roundtripped = fs::read(&reassembled).unwrap();
  assert_eq!(
    original, roundtripped,
    "reassembled bytecode should be byte-identical to original"
  );
}

#[test]
fn file_too_small() {
  let tmp = tempfile::tempdir().unwrap();
  let bad = tmp.path().join("tiny.hcb");
  fs::write(&bad, &[0u8; 2]).unwrap();
  let result = hime_tools::hcb::disasm(&bad, &tmp.path().join("out"));
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("too small"));
}

#[test]
fn descriptor_offset_exceeds_file() {
  let tmp = tempfile::tempdir().unwrap();
  let bad = tmp.path().join("bad_offset.hcb");
  let mut data = vec![0xFF, 0xFF, 0xFF, 0xFF];
  data.extend_from_slice(&[0u8; 8]);
  fs::write(&bad, &data).unwrap();
  let result = hime_tools::hcb::disasm(&bad, &tmp.path().join("out"));
  assert!(result.is_err());
  let err = result.unwrap_err().to_string();
  assert!(err.contains("descriptor offset"), "error was: {}", err);
}
