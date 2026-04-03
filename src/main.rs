use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use hime_tools::{archive, hcb, nvsg};

mod cli;
use cli::*;

fn main() -> Result<()> {
  let cli = Cli::parse();
  match cli.command {
    Command::Bin { action } => match action {
      BinAction::Ls { archive } => archive::ls(&archive),
      BinAction::Unpack { archive, output_folder } => archive::unpack(&archive, &output_folder),
      BinAction::Get { archive, name, output } => {
        let output = output.unwrap_or_else(|| PathBuf::from(&name));
        archive::get(&archive, &name, &output)
      }
      BinAction::Pack { input_folder, output } => archive::pack(&input_folder, &output),
      BinAction::Validate { archive } => archive::validate(&archive),
      BinAction::Replace { archive, name, file } => archive::replace(&archive, &name, &file),
    },
    Command::Hcb { action } => match action {
      HcbAction::Disasm { input, output } => hcb::disasm(&input, &output),
      HcbAction::Asm { input, output } => hcb::asm(&input, &output),
    },
    Command::Nvsg { action } => match action {
      NvsgAction::Info { input } => nvsg::info(&input),
      NvsgAction::Decode { input, output } => {
        let output = if input.is_dir() {
          output.ok_or_else(|| anyhow::anyhow!("output directory is required when input is a directory"))?
        } else {
          output.unwrap_or_else(|| input.with_extension("png"))
        };
        nvsg::decode(&input, &output)
      }
      NvsgAction::Encode {
        input,
        output,
        offset_x,
        offset_y,
        anchor_x,
        anchor_y,
        parts_count,
        image_type,
      } => {
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
