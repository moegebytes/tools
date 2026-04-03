pub fn parse_int<T: std::str::FromStr + TryFrom<i64>>(s: &str) -> Option<T> {
  if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
    i64::from_str_radix(hex, 16).ok().and_then(|v| T::try_from(v).ok())
  } else {
    s.parse::<T>().ok()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decimal_i32() {
    assert_eq!(parse_int::<i32>("0"), Some(0));
    assert_eq!(parse_int::<i32>("42"), Some(42));
    assert_eq!(parse_int::<i32>("-1"), Some(-1));
  }

  #[test]
  fn decimal_u32() {
    assert_eq!(parse_int::<u32>("0"), Some(0));
    assert_eq!(parse_int::<u32>("65535"), Some(65535));
  }

  #[test]
  fn hex_lowercase() {
    assert_eq!(parse_int::<u32>("0xFF"), Some(255));
    assert_eq!(parse_int::<u32>("0x0"), Some(0));
    assert_eq!(parse_int::<i32>("0x1a"), Some(26));
  }

  #[test]
  fn hex_uppercase_prefix() {
    assert_eq!(parse_int::<u32>("0XFF"), Some(255));
    assert_eq!(parse_int::<i32>("0X1A"), Some(26));
  }

  #[test]
  fn u8_in_range() {
    assert_eq!(parse_int::<u8>("0"), Some(0));
    assert_eq!(parse_int::<u8>("255"), Some(255));
    assert_eq!(parse_int::<u8>("0xFF"), Some(255));
  }

  #[test]
  fn u8_overflow() {
    assert_eq!(parse_int::<u8>("256"), None);
    assert_eq!(parse_int::<u8>("0x100"), None);
  }

  #[test]
  fn negative_unsigned_fails() {
    assert_eq!(parse_int::<u32>("-1"), None);
  }

  #[test]
  fn invalid_input() {
    assert_eq!(parse_int::<i32>(""), None);
    assert_eq!(parse_int::<i32>("abc"), None);
    assert_eq!(parse_int::<i32>("0xGG"), None);
  }
}
