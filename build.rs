use std::io::Error;

fn main() -> Result<(), Error> {
  #[cfg(target_os = "windows")]
  {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("assets/app.manifest");
    res.compile()?;
  }

  Ok(())
}
