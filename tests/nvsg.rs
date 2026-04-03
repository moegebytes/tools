use std::fs;
use std::path::{Path, PathBuf};

use hime_tools::nvsg::{EncodeOptions, ImageType};
use hime_tools::utils::png::load_png;

fn fixtures() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn decode_gaiji() {
  let tmp = tempfile::tempdir().unwrap();
  let out_png = tmp.path().join("gaiji_heart156.png");

  hime_tools::nvsg::decode(&fixtures().join("gaiji_heart156.nvsg"), &out_png).unwrap();

  let (color, w, h, _) = load_png(&out_png).unwrap();
  assert_eq!(w, 234);
  assert_eq!(h, 234);
  assert_eq!(color, png::ColorType::Rgba);
}

#[test]
fn decode_bgra() {
  let tmp = tempfile::tempdir().unwrap();
  let out_png = tmp.path().join("x禁止.png");

  hime_tools::nvsg::decode(&fixtures().join("x禁止.nvsg"), &out_png).unwrap();

  let (color, w, h, _) = load_png(&out_png).unwrap();
  assert_eq!(w, 139);
  assert_eq!(h, 34);
  assert_eq!(color, png::ColorType::Rgba);
}

#[test]
fn decode_parts() {
  let tmp = tempfile::tempdir().unwrap();
  let out_png = tmp.path().join("parts.png");

  hime_tools::nvsg::decode(&fixtures().join("CHR_メア_悲_死神カマ_表情.nvsg"), &out_png).unwrap();

  let (color, w, h, _) = load_png(&out_png).unwrap();
  assert_eq!(w, 184);
  assert_eq!(h, 164 * 20, "height should be frame_height * parts_count");
  assert_eq!(color, png::ColorType::Rgba);
}

#[test]
fn roundtrip_parts_pixels() {
  let tmp = tempfile::tempdir().unwrap();
  let png1 = tmp.path().join("step1.png");
  let reencoded = tmp.path().join("step2.nvsg");
  let png2 = tmp.path().join("step3.png");

  hime_tools::nvsg::decode(&fixtures().join("CHR_メア_悲_死神カマ_表情.nvsg"), &png1).unwrap();

  let opts = EncodeOptions {
    image_type: Some(ImageType::Parts),
    offset_x: 700,
    offset_y: 172,
    anchor_x: 0,
    anchor_y: 0,
    parts_count: 20,
  };

  hime_tools::nvsg::encode(&png1, &reencoded, &opts).unwrap();
  hime_tools::nvsg::decode(&reencoded, &png2).unwrap();

  let (_, _, _, data1) = load_png(&png1).unwrap();
  let (_, _, _, data2) = load_png(&png2).unwrap();
  assert_eq!(data1, data2, "parts pixel roundtrip should be lossless");
}

#[test]
fn decode_dir_produces_pngs() {
  let input_dir = tempfile::tempdir().unwrap();
  let output_dir = tempfile::tempdir().unwrap();

  fs::copy(
    fixtures().join("gaiji_heart156.nvsg"),
    input_dir.path().join("gaiji_heart156.nvsg"),
  )
  .unwrap();
  fs::copy(fixtures().join("x禁止.nvsg"), input_dir.path().join("x禁止.nvsg")).unwrap();
  fs::copy(
    fixtures().join("CHR_メア_悲_死神カマ_表情.nvsg"),
    input_dir.path().join("CHR_メア_悲_死神カマ_表情.nvsg"),
  )
  .unwrap();

  hime_tools::nvsg::decode(input_dir.path(), output_dir.path()).unwrap();

  let (_, w1, h1, _) = load_png(&output_dir.path().join("gaiji_heart156.png")).unwrap();
  assert_eq!((w1, h1), (234, 234));

  let (_, w2, h2, _) = load_png(&output_dir.path().join("x禁止.png")).unwrap();
  assert_eq!((w2, h2), (139, 34));

  let (_, w2, h2, _) = load_png(&output_dir.path().join("CHR_メア_悲_死神カマ_表情.png")).unwrap();
  assert_eq!((w2, h2), (184, 3280));
}

#[test]
fn roundtrip_gaiji_pixels() {
  let tmp = tempfile::tempdir().unwrap();
  let png1 = tmp.path().join("step1.png");
  let reencoded = tmp.path().join("step2.nvsg");
  let png2 = tmp.path().join("step3.png");

  hime_tools::nvsg::decode(&fixtures().join("gaiji_heart156.nvsg"), &png1).unwrap();

  let opts = EncodeOptions {
    image_type: Some(ImageType::Gaiji),
    offset_x: 0,
    offset_y: 0,
    anchor_x: 0,
    anchor_y: 0,
    parts_count: 0,
  };

  hime_tools::nvsg::encode(&png1, &reencoded, &opts).unwrap();
  hime_tools::nvsg::decode(&reencoded, &png2).unwrap();

  let (_, _, _, data1) = load_png(&png1).unwrap();
  let (_, _, _, data2) = load_png(&png2).unwrap();
  assert_eq!(data1, data2, "gaiji pixel roundtrip should be lossless");
}

#[test]
fn roundtrip_bgra_pixels() {
  let tmp = tempfile::tempdir().unwrap();
  let png1 = tmp.path().join("step1.png");
  let reencoded = tmp.path().join("step2.nvsg");
  let png2 = tmp.path().join("step3.png");

  hime_tools::nvsg::decode(&fixtures().join("x禁止.nvsg"), &png1).unwrap();

  let opts = EncodeOptions {
    image_type: Some(ImageType::Bgra),
    offset_x: 891,
    offset_y: 1036,
    anchor_x: 0,
    anchor_y: 0,
    parts_count: 0,
  };

  hime_tools::nvsg::encode(&png1, &reencoded, &opts).unwrap();
  hime_tools::nvsg::decode(&reencoded, &png2).unwrap();

  let (_, _, _, data1) = load_png(&png1).unwrap();
  let (_, _, _, data2) = load_png(&png2).unwrap();
  assert_eq!(data1, data2, "bgra pixel roundtrip should be lossless");
}

#[test]
fn bad_magic() {
  let tmp = tempfile::tempdir().unwrap();
  let bad_file = tmp.path().join("bad.nvsg");

  let mut data = Vec::new();
  data.extend_from_slice(b"hzc1");
  data.extend_from_slice(&0u32.to_le_bytes());
  data.extend_from_slice(&32u32.to_le_bytes());
  data.extend_from_slice(b"junk");
  data.extend_from_slice(&[0u8; 28]);

  let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
  std::io::Write::write_all(&mut encoder, &[]).unwrap();
  data.extend_from_slice(&encoder.finish().unwrap());
  fs::write(&bad_file, &data).unwrap();

  let result = hime_tools::nvsg::decode(&bad_file, &tmp.path().join("out.png"));
  assert!(result.is_err());
  let err = result.unwrap_err().to_string();
  assert!(err.contains("not an nvsg file"), "error was: {}", err);
}
