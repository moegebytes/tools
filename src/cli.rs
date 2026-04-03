use std::path::PathBuf;

use argh::FromArgs;

/// Tools for translating games using NVS/FVP visual novel engine
#[derive(FromArgs)]
pub struct Cli {
  #[argh(subcommand)]
  pub command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Command {
  Bin(BinCommand),
  Nvsg(NvsgCommand),
  Hcb(HcbCommand),
}

/// Packed archive operations
#[derive(FromArgs)]
#[argh(subcommand, name = "bin")]
pub struct BinCommand {
  #[argh(subcommand)]
  pub action: BinAction,
}

/// NVSG image operations
#[derive(FromArgs)]
#[argh(subcommand, name = "nvsg")]
pub struct NvsgCommand {
  #[argh(subcommand)]
  pub action: NvsgAction,
}

/// HCB bytecode operations
#[derive(FromArgs)]
#[argh(subcommand, name = "hcb")]
pub struct HcbCommand {
  #[argh(subcommand)]
  pub action: HcbAction,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum BinAction {
  Ls(BinLs),
  Get(BinGet),
  Pack(BinPack),
  Unpack(BinUnpack),
  Validate(BinValidate),
  Replace(BinReplace),
}

/// List archive contents
#[derive(FromArgs)]
#[argh(subcommand, name = "ls")]
pub struct BinLs {
  #[argh(positional)]
  pub archive: PathBuf,
}

/// Extract a single file from archive
#[derive(FromArgs)]
#[argh(subcommand, name = "get")]
pub struct BinGet {
  #[argh(positional)]
  pub archive: PathBuf,
  #[argh(positional)]
  pub name: String,
  #[argh(positional)]
  pub output: Option<PathBuf>,
}

/// Pack a folder into an archive
#[derive(FromArgs)]
#[argh(subcommand, name = "pack")]
pub struct BinPack {
  #[argh(positional)]
  pub input_folder: PathBuf,
  #[argh(positional)]
  pub output: PathBuf,
}

/// Unpack an archive into a folder
#[derive(FromArgs)]
#[argh(subcommand, name = "unpack")]
pub struct BinUnpack {
  #[argh(positional)]
  pub archive: PathBuf,
  #[argh(positional)]
  pub output_folder: PathBuf,
}

/// Validate archive integrity
#[derive(FromArgs)]
#[argh(subcommand, name = "validate")]
pub struct BinValidate {
  #[argh(positional)]
  pub archive: PathBuf,
}

/// Replace a file in an archive
#[derive(FromArgs)]
#[argh(subcommand, name = "replace")]
pub struct BinReplace {
  #[argh(positional)]
  pub archive: PathBuf,
  #[argh(positional)]
  pub name: String,
  #[argh(positional)]
  pub file: PathBuf,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum HcbAction {
  Disasm(HcbDisasm),
  Asm(HcbAsm),
}

/// Disassemble HCB bytecode
#[derive(FromArgs)]
#[argh(subcommand, name = "disasm")]
pub struct HcbDisasm {
  #[argh(positional)]
  pub input: PathBuf,
  #[argh(positional)]
  pub output: PathBuf,
}

/// Assemble HCB bytecode
#[derive(FromArgs)]
#[argh(subcommand, name = "asm")]
pub struct HcbAsm {
  #[argh(positional)]
  pub input: PathBuf,
  #[argh(positional)]
  pub output: PathBuf,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum NvsgAction {
  Info(NvsgInfo),
  Decode(NvsgDecode),
  Encode(NvsgEncode),
}

/// Show NVSG image info
#[derive(FromArgs)]
#[argh(subcommand, name = "info")]
pub struct NvsgInfo {
  #[argh(positional)]
  pub input: PathBuf,
}

/// Decode NVSG image to PNG
#[derive(FromArgs)]
#[argh(subcommand, name = "decode")]
pub struct NvsgDecode {
  #[argh(positional)]
  pub input: PathBuf,
  #[argh(positional)]
  pub output: Option<PathBuf>,
}

/// Encode PNG to NVSG image
#[derive(FromArgs)]
#[argh(subcommand, name = "encode")]
pub struct NvsgEncode {
  #[argh(positional)]
  pub input: PathBuf,
  #[argh(positional)]
  pub output: PathBuf,
  #[argh(option, short = 'x', default = "0", description = "horizontal offset")]
  pub offset_x: u16,
  #[argh(option, short = 'y', default = "0", description = "vertical offset")]
  pub offset_y: u16,
  #[argh(option, short = 'u', default = "0", description = "horizontal anchor")]
  pub anchor_x: u16,
  #[argh(option, short = 'v', default = "0", description = "vertical anchor")]
  pub anchor_y: u16,
  #[argh(option, long = "type", description = "image type")]
  pub image_type: Option<u16>,
  #[argh(option, long = "parts", default = "0", description = "number of parts")]
  pub parts_count: u16,
}
