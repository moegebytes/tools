use std::fs;

#[test]
fn load_with_includes() {
  let tmp = tempfile::tempdir().unwrap();

  fs::write(tmp.path().join("nested.txt"), "from nested\n").unwrap();
  fs::write(
    tmp.path().join("extra.txt"),
    "from extra before\n#include <nested.txt>\nfrom extra after\n",
  )
  .unwrap();
  fs::write(tmp.path().join("strings.txt"), "first\n#include <extra.txt>\nlast\n").unwrap();

  let strings = hime_tools::utils::strings::load(&tmp.path().join("strings.txt")).unwrap();
  assert_eq!(
    strings,
    vec!["first", "from extra before", "from nested", "from extra after", "last"]
  );
}

#[test]
fn load_cycle_detected() {
  let tmp = tempfile::tempdir().unwrap();
  fs::write(tmp.path().join("a.txt"), "a\n#include <b.txt>\n").unwrap();
  fs::write(tmp.path().join("b.txt"), "b\n#include <a.txt>\n").unwrap();

  let result = hime_tools::utils::strings::load(&tmp.path().join("a.txt"));
  assert!(result.is_err());
  let err = result.unwrap_err();
  let chain: String = err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join(": ");
  assert!(chain.contains("circular include"), "error was: {}", chain);
}

#[test]
fn load_no_includes() {
  let tmp = tempfile::tempdir().unwrap();
  fs::write(tmp.path().join("strings.txt"), "hello\nworld\n").unwrap();

  let strings = hime_tools::utils::strings::load(&tmp.path().join("strings.txt")).unwrap();
  assert_eq!(strings, vec!["hello", "world"]);
}

#[test]
fn load_emit_empty() {
  let tmp = tempfile::tempdir().unwrap();
  fs::write(tmp.path().join("strings.txt"), "first\n#emit empty\nlast\n").unwrap();

  let strings = hime_tools::utils::strings::load(&tmp.path().join("strings.txt")).unwrap();
  assert_eq!(strings, vec!["first", "", "last"]);
}

#[test]
fn load_comments_and_blanks() {
  let tmp = tempfile::tempdir().unwrap();
  fs::write(
    tmp.path().join("strings.txt"),
    "first\n\n; this is a comment\n; another comment\n\nlast\n",
  )
  .unwrap();

  let strings = hime_tools::utils::strings::load(&tmp.path().join("strings.txt")).unwrap();
  assert_eq!(strings, vec!["first", "last"]);
}

#[test]
fn load_reference_skipped() {
  let tmp = tempfile::tempdir().unwrap();
  fs::write(
    tmp.path().join("strings.txt"),
    "#reference <some.txt>\nfirst\n#reference <other.txt>\nlast\n",
  )
  .unwrap();

  let strings = hime_tools::utils::strings::load(&tmp.path().join("strings.txt")).unwrap();
  assert_eq!(strings, vec!["first", "last"]);
}
