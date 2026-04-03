use std::fs;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn unpack_graph_bin() {
  let tmp = tempfile::tempdir().unwrap();
  hime_tools::archive::unpack(&fixtures().join("graph.bin"), tmp.path()).unwrap();

  let mut files: Vec<String> = fs::read_dir(tmp.path())
    .unwrap()
    .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
    .collect();
  files.sort();
  assert_eq!(files.len(), 3);
  assert_eq!(files[0], "CHR_メア_悲_死神カマ_表情");
  assert_eq!(files[1], "gaiji_heart156");
  assert_eq!(files[2], "x禁止");

  let extracted = fs::read(tmp.path().join("gaiji_heart156")).unwrap();
  let original = fs::read(fixtures().join("gaiji_heart156.nvsg")).unwrap();
  assert_eq!(
    extracted, original,
    "extracted gaiji_heart156 should match gaiji_heart156.nvsg"
  );

  let extracted = fs::read(tmp.path().join("x禁止")).unwrap();
  let original = fs::read(fixtures().join("x禁止.nvsg")).unwrap();
  assert_eq!(extracted, original, "extracted x禁止 should match x禁止.nvsg");

  let extracted = fs::read(tmp.path().join("CHR_メア_悲_死神カマ_表情")).unwrap();
  let original = fs::read(fixtures().join("CHR_メア_悲_死神カマ_表情.nvsg")).unwrap();
  assert_eq!(
    extracted, original,
    "extracted CHR_メア_悲_死神カマ_表情 should match CHR_メア_悲_死神カマ_表情.nvsg"
  );
}

#[test]
fn get_single_file() {
  let tmp = tempfile::tempdir().unwrap();
  let out = tmp.path().join("gaiji_heart");

  hime_tools::archive::get(&fixtures().join("graph.bin"), "gaiji_heart156", &out).unwrap();

  let extracted = fs::read(&out).unwrap();
  let original = fs::read(fixtures().join("gaiji_heart156.nvsg")).unwrap();
  assert_eq!(extracted, original);
}

#[test]
fn get_single_file_case_insensitive() {
  let tmp = tempfile::tempdir().unwrap();
  let out = tmp.path().join("gaiji_heart");

  hime_tools::archive::get(&fixtures().join("graph.bin"), "GAIJI_HEART156", &out).unwrap();

  let extracted = fs::read(&out).unwrap();
  let original = fs::read(fixtures().join("gaiji_heart156.nvsg")).unwrap();
  assert_eq!(extracted, original);
}

#[test]
fn get_missing_file_errors() {
  let tmp = tempfile::tempdir().unwrap();
  let result = hime_tools::archive::get(&fixtures().join("graph.bin"), "nonexistent", &tmp.path().join("out"));
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn replace_single_file() {
  let tmp = tempfile::tempdir().unwrap();
  let archive = tmp.path().join("graph.bin");
  fs::copy(fixtures().join("graph.bin"), &archive).unwrap();

  let new_content = b"replaced content";
  let replacement = tmp.path().join("replacement");
  fs::write(&replacement, new_content).unwrap();

  hime_tools::archive::replace(&archive, "gaiji_heart156", &replacement).unwrap();

  let out = tmp.path().join("extracted");
  hime_tools::archive::get(&archive, "gaiji_heart156", &out).unwrap();
  assert_eq!(fs::read(&out).unwrap(), new_content);

  // Other entries should be unaffected
  let out2 = tmp.path().join("other");
  hime_tools::archive::get(&archive, "x禁止", &out2).unwrap();
  let original = fs::read(fixtures().join("x禁止.nvsg")).unwrap();
  assert_eq!(fs::read(&out2).unwrap(), original);
}

#[test]
fn replace_missing_file_errors() {
  let tmp = tempfile::tempdir().unwrap();
  let archive = tmp.path().join("graph.bin");
  fs::copy(fixtures().join("graph.bin"), &archive).unwrap();

  let replacement = tmp.path().join("replacement");
  fs::write(&replacement, b"data").unwrap();

  let result = hime_tools::archive::replace(&archive, "nonexistent", &replacement);
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn roundtrip_unpack_pack() {
  let unpack_dir = tempfile::tempdir().unwrap();
  let repack_dir = tempfile::tempdir().unwrap();

  hime_tools::archive::unpack(&fixtures().join("graph.bin"), unpack_dir.path()).unwrap();

  let repacked = repack_dir.path().join("repacked.bin");
  hime_tools::archive::pack(unpack_dir.path(), &repacked).unwrap();

  let original = fs::read(fixtures().join("graph.bin")).unwrap();
  let repacked_data = fs::read(&repacked).unwrap();
  assert_eq!(
    original, repacked_data,
    "repacked archive should be byte-identical to original"
  );
}
