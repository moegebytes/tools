use std::path::PathBuf;

use anyhow::Result;
use hime_tools::{archive, hcb, nvsg};

mod cli;
use cli::*;

fn main() -> Result<()> {
  let cli: Cli = argh::from_env();
  match cli.command {
    Command::Bin(BinCommand { action }) => match action {
      BinAction::Ls(BinLs { archive }) => archive::ls(&archive),
      BinAction::Unpack(BinUnpack { archive, output_folder }) => archive::unpack(&archive, &output_folder),
      BinAction::Get(BinGet { archive, name, output }) => {
        let output = output.unwrap_or_else(|| PathBuf::from(&name));
        archive::get(&archive, &name, &output)
      }
      BinAction::Pack(BinPack { input_folder, output }) => archive::pack(&input_folder, &output),
      BinAction::Validate(BinValidate { archive }) => archive::validate(&archive),
      BinAction::Replace(BinReplace { archive, name, file }) => archive::replace(&archive, &name, &file),
    },
    Command::Hcb(HcbCommand { action }) => match action {
      HcbAction::Disasm(HcbDisasm { input, output }) => hcb::disasm(&input, &output),
      HcbAction::Asm(HcbAsm { input, output }) => hcb::asm(&input, &output),
    },
    Command::Nvsg(NvsgCommand { action }) => match action {
      NvsgAction::Info(NvsgInfo { input }) => nvsg::info(&input),
      NvsgAction::Decode(NvsgDecode { input, output }) => {
        let output = if input.is_dir() {
          output.ok_or_else(|| anyhow::anyhow!("output directory is required when input is a directory"))?
        } else {
          output.unwrap_or_else(|| input.with_extension("png"))
        };
        nvsg::decode(&input, &output)
      }
      NvsgAction::Encode(NvsgEncode {
        input,
        output,
        offset_x,
        offset_y,
        anchor_x,
        anchor_y,
        parts_count,
        image_type,
      }) => {
        let image_type = image_type.map(nvsg::ImageType::try_from).transpose()?;
        nvsg::encode(
          &input,
          &output,
          &nvsg::EncodeOptions {
            image_type,
            offset_x,
            offset_y,
            anchor_x,
            anchor_y,
            parts_count,
          },
        )
      }
    },
  }
}
