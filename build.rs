use std::io::Error;

use clap::CommandFactory;
use clap::ValueEnum;
use clap_complete::{generate_to, Shell};

#[path = "src/cli.rs"]
mod cli;

fn main() -> Result<(), Error> {
  #[cfg(target_os = "windows")]
  {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("assets/app.manifest");
    res.compile()?;
  }

  let mut cmd = cli::Cli::command();
  for &shell in Shell::value_variants() {
    generate_to(shell, &mut cmd, "hime-tools", "completions")?;
  }

  Ok(())
}
