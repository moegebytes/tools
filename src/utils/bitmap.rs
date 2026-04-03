pub fn swap_bgr_rgb(data: &mut [u8], bpp: usize) {
  for chunk in data.chunks_exact_mut(bpp) {
    chunk.swap(0, 2);
  }
}

pub fn rgba_from_rgb(data: &[u8]) -> Vec<u8> {
  let mut out = Vec::with_capacity(data.len() / 3 * 4);
  for rgb in data.chunks_exact(3) {
    out.extend_from_slice(rgb);
    out.push(0xFF);
  }
  out
}

pub fn rgb_from_rgba(data: &[u8]) -> Vec<u8> {
  let mut out = Vec::with_capacity(data.len() / 4 * 3);
  for rgba in data.chunks_exact(4) {
    out.extend_from_slice(&rgba[..3]);
  }
  out
}

pub fn grayscale_from_rgb(data: &[u8]) -> Vec<u8> {
  data
    .chunks_exact(3)
    .map(|rgb| ((rgb[0] as u16 * 77 + rgb[1] as u16 * 150 + rgb[2] as u16 * 29) >> 8) as u8)
    .collect()
}

pub fn rgba_from_mask(data: &[u8]) -> Vec<u8> {
  let mut out = Vec::with_capacity(data.len() * 4);
  for &v in data {
    let a = if v > 0 { 0xFF } else { 0 };
    out.extend_from_slice(&[0xFF, 0xFF, 0xFF, a]);
  }
  out
}

pub fn mask_from_rgba(data: &[u8]) -> Vec<u8> {
  data
    .chunks_exact(4)
    .map(|rgba| if rgba[3] >= 128 { 1u8 } else { 0u8 })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn swap_bgr_rgb_3bpp() {
    let mut data = vec![0, 1, 2, 3, 4, 5];
    swap_bgr_rgb(&mut data, 3);
    assert_eq!(data, [2, 1, 0, 5, 4, 3]);
  }

  #[test]
  fn swap_bgr_rgb_4bpp() {
    let mut data = vec![0, 1, 2, 0xFF, 3, 4, 5, 0xFE];
    swap_bgr_rgb(&mut data, 4);
    assert_eq!(data, [2, 1, 0, 0xFF, 5, 4, 3, 0xFE]);
  }

  #[test]
  fn swap_bgr_rgb_roundtrip() {
    let original = vec![10, 20, 30, 40, 50, 60];
    let mut data = original.clone();
    swap_bgr_rgb(&mut data, 3);
    swap_bgr_rgb(&mut data, 3);
    assert_eq!(data, original);
  }

  #[test]
  fn rgba_from_rgb_adds_opaque_alpha() {
    let rgb = vec![10, 20, 30, 40, 50, 60];
    let rgba = rgba_from_rgb(&rgb);
    assert_eq!(rgba, [10, 20, 30, 0xFF, 40, 50, 60, 0xFF]);
  }

  #[test]
  fn rgb_from_rgba_strips_alpha() {
    let rgba = vec![10, 20, 30, 0xFF, 40, 50, 60, 0x80];
    let rgb = rgb_from_rgba(&rgba);
    assert_eq!(rgb, [10, 20, 30, 40, 50, 60]);
  }

  #[test]
  fn rgb_rgba_roundtrip() {
    let original = vec![1, 2, 3, 4, 5, 6];
    let roundtripped = rgb_from_rgba(&rgba_from_rgb(&original));
    assert_eq!(roundtripped, original);
  }

  #[test]
  fn grayscale_from_rgb_weighted_average() {
    let gray = grayscale_from_rgb(&[255, 0, 0]);
    assert_eq!(gray, [(255u16 * 77 >> 8) as u8]);

    let gray = grayscale_from_rgb(&[255, 255, 255]);
    assert_eq!(gray, [255]);

    let gray = grayscale_from_rgb(&[0, 0, 0]);
    assert_eq!(gray, [0]);
  }

  #[test]
  fn mask_from_rgba_threshold() {
    let data = vec![
      0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 128, 0xFF, 0xFF, 0xFF, 127, 0xFF, 0xFF, 0xFF, 0,
    ];
    assert_eq!(mask_from_rgba(&data), [1, 1, 0, 0]);
  }

  #[test]
  fn rgba_from_mask_binary() {
    let mask = vec![0, 1, 255];
    let rgba = rgba_from_mask(&mask);
    assert_eq!(
      rgba,
      [0xFF, 0xFF, 0xFF, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,]
    );
  }
}
