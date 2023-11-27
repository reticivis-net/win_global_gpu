use std::{env, io};
use winres::WindowsResource;
fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("win_global_gpu.ico")
            .compile()?;
    }
    Ok(())
}
