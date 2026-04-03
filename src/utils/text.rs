use std::cmp::Ordering;

use anyhow::{bail, Result};

pub fn strip_nul(bytes: &[u8]) -> &[u8] {
  bytes.strip_suffix(&[0]).unwrap_or(bytes)
}

pub fn decode_sjis(bytes: &[u8]) -> String {
  let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(strip_nul(bytes));
  decoded.into_owned()
}

pub fn encode_sjis(s: &str) -> Result<Vec<u8>> {
  let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(s);
  if had_errors {
    bail!("failed to encode string as sjis: {:?}", s);
  }
  let mut bytes = encoded.into_owned();
  bytes.push(0); // nul terminator
  Ok(bytes)
}

pub fn encode_sjis_lossy(s: &str) -> (Vec<u8>, bool) {
  let (encoded, _, had_errors) = encoding_rs::SHIFT_JIS.encode(s);
  (encoded.into_owned(), had_errors)
}

pub fn decode_sjis_lossy(bytes: &[u8]) -> (String, bool) {
  let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(bytes);
  (decoded.into_owned(), had_errors)
}

pub fn cmp_sjis_case_insensitive(a: &[u8], b: &[u8]) -> Ordering {
  let mut ai = 0;
  let mut bi = 0;
  while ai < a.len() && bi < b.len() {
    let (aw, a_step) = sjis_sort_key(a, ai);
    let (bw, b_step) = sjis_sort_key(b, bi);
    match aw.cmp(&bw) {
      Ordering::Equal => {
        ai += a_step;
        bi += b_step;
      }
      ord => return ord,
    }
  }
  a.len().cmp(&b.len())
}

fn sjis_sort_key(data: &[u8], pos: usize) -> (u32, usize) {
  let b = data[pos];
  if is_sjis_lead(b) && pos + 1 < data.len() {
    (0x10000 | ((b as u32) << 8) | data[pos + 1] as u32, 2)
  } else {
    (ascii_sort_weight(b) as u32, 1)
  }
}

fn ascii_sort_weight(b: u8) -> u16 {
  match b {
    b'A'..=b'Z' => 0x100 + (b - b'A') as u16,
    b'a'..=b'z' => 0x100 + (b - b'a') as u16,
    b'0'..=b'9' => 0x80 + (b - b'0') as u16,
    _ => b as u16,
  }
}

fn is_sjis_lead(b: u8) -> bool {
  (0x81..=0x9F).contains(&b) || (0xE0..=0xEF).contains(&b)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sjis_lead_byte_ranges() {
    assert!(is_sjis_lead(0x81));
    assert!(is_sjis_lead(0x9F));
    assert!(is_sjis_lead(0xE0));
    assert!(is_sjis_lead(0xEF));

    assert!(!is_sjis_lead(0x80));
    assert!(!is_sjis_lead(0xA0));
    assert!(!is_sjis_lead(0xDF));
    assert!(!is_sjis_lead(0xF0));
    assert!(!is_sjis_lead(b'A'));
  }

  #[test]
  fn ascii_weight_case_insensitive() {
    assert_eq!(ascii_sort_weight(b'A'), ascii_sort_weight(b'a'));
    assert_eq!(ascii_sort_weight(b'Z'), ascii_sort_weight(b'z'));
    assert_eq!(ascii_sort_weight(b'M'), ascii_sort_weight(b'm'));
  }

  #[test]
  fn ascii_weight_ordering_symbols_before_digits_before_letters() {
    assert!(ascii_sort_weight(b'!') < ascii_sort_weight(b'0'));
    assert!(ascii_sort_weight(b'.') < ascii_sort_weight(b'0'));
    assert!(ascii_sort_weight(b'0') < ascii_sort_weight(b'a'));
    assert!(ascii_sort_weight(b'9') < ascii_sort_weight(b'A'));
  }

  #[test]
  fn ascii_weight_digit_order() {
    for w in b'0'..b'9' {
      assert!(ascii_sort_weight(w) < ascii_sort_weight(w + 1));
    }
  }

  #[test]
  fn ascii_weight_letter_order() {
    for w in b'A'..b'Z' {
      assert!(ascii_sort_weight(w) < ascii_sort_weight(w + 1));
    }
    for w in b'a'..b'z' {
      assert!(ascii_sort_weight(w) < ascii_sort_weight(w + 1));
    }
  }

  #[test]
  fn sort_key_single_byte_ascii() {
    let data = b"A";
    let (key, step) = sjis_sort_key(data, 0);
    assert_eq!(step, 1);
    assert_eq!(key, ascii_sort_weight(b'A') as u32);
  }

  #[test]
  fn sort_key_multibyte_sjis() {
    let data = [0x8B, 0xD6]; // 禁
    let (key, step) = sjis_sort_key(&data, 0);
    assert_eq!(step, 2);
    assert!(key >= 0x10000, "sjis multibyte key should be in high range");
    assert_eq!(key, 0x10000 | (0x8B << 8) | 0xD6);
  }

  #[test]
  fn cmp_case_insensitive_equal() {
    assert_eq!(cmp_sjis_case_insensitive(b"abc", b"ABC"), Ordering::Equal);
    assert_eq!(cmp_sjis_case_insensitive(b"Hello", b"hELLO"), Ordering::Equal);
  }

  #[test]
  fn cmp_digits_before_letters() {
    assert_eq!(cmp_sjis_case_insensitive(b"1file", b"afile"), Ordering::Less);
    assert_eq!(cmp_sjis_case_insensitive(b"9", b"a"), Ordering::Less);
  }

  #[test]
  fn cmp_symbols_before_digits() {
    assert_eq!(cmp_sjis_case_insensitive(b"!file", b"0file"), Ordering::Less);
    assert_eq!(cmp_sjis_case_insensitive(b"_", b"0"), Ordering::Less);
  }

  #[test]
  fn cmp_sjis_multibyte_after_ascii() {
    let ascii = b"z";
    let sjis = [0x8B, 0xD6]; // 禁
    assert_eq!(cmp_sjis_case_insensitive(ascii, &sjis), Ordering::Less);
  }

  #[test]
  fn cmp_prefix_shorter_is_less() {
    assert_eq!(cmp_sjis_case_insensitive(b"abc", b"abcd"), Ordering::Less);
    assert_eq!(cmp_sjis_case_insensitive(b"abcd", b"abc"), Ordering::Greater);
  }

  #[test]
  fn sort_matches_engine_order() {
    let mut names: Vec<&[u8]> = vec![
      b"bg_01",
      b"_system",
      b"alpha",
      b"ALPHA",
      b"0test",
      &[0x8B, 0xD6], // 禁
    ];
    names.sort_by(|a, b| cmp_sjis_case_insensitive(a, b));

    let sorted: Vec<&[u8]> = vec![b"_system", b"0test", b"alpha", b"ALPHA", b"bg_01", &[0x8B, 0xD6]];
    assert_eq!(names, sorted);
  }
}
