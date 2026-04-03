use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
  name = "hime-tools",
  about = "Tools for translating games using NVS/FVP visual novel engine"
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  Bin {
    #[command(subcommand)]
    action: BinAction,
  },
  Nvsg {
    #[command(subcommand)]
    action: NvsgAction,
  },
  Hcb {
    #[command(subcommand)]
    action: HcbAction,
  },
}

#[derive(Subcommand)]
pub enum BinAction {
  Ls {
    archive: PathBuf,
  },
  Get {
    archive: PathBuf,
    name: String,
    output: Option<PathBuf>,
  },
  Pack {
    input_folder: PathBuf,
    output: PathBuf,
  },
  Unpack {
    archive: PathBuf,
    output_folder: PathBuf,
  },
  Validate {
    archive: PathBuf,
  },
  Replace {
    archive: PathBuf,
    name: String,
    file: PathBuf,
  },
}

#[derive(Subcommand)]
pub enum HcbAction {
  Disasm { input: PathBuf, output: PathBuf },
  Asm { input: PathBuf, output: PathBuf },
}

#[derive(Subcommand)]
pub enum NvsgAction {
  Info {
    input: PathBuf,
  },
  Decode {
    input: PathBuf,
    output: Option<PathBuf>,
  },
  Encode {
    input: PathBuf,
    output: PathBuf,
    #[arg(short = 'x', default_value_t = 0)]
    offset_x: u16,
    #[arg(short = 'y', default_value_t = 0)]
    offset_y: u16,
    #[arg(short = 'u', default_value_t = 0)]
    anchor_x: u16,
    #[arg(short = 'v', default_value_t = 0)]
    anchor_y: u16,
    #[arg(long = "type")]
    image_type: Option<u16>,
    #[arg(long = "parts", default_value_t = 0)]
    parts_count: u16,
  },
}
