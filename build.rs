use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        // Only run this on Windows
        // Check if the icon file exists to avoid build errors on other/clean systems
        if std::path::Path::new("logo.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("logo.ico");
            res.compile()?;
        }
    }
    Ok(())
}
