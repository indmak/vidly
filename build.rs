fn main() {
    // Use CARGO_CFG_TARGET_OS (not cfg!) so cross-compilation is handled correctly.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let icon = "assets/vidly.ico";
    println!("cargo:rerun-if-changed={icon}");
    if !std::path::Path::new(icon).exists() {
        println!("cargo:warning=assets/vidly.ico not found; skipping Windows icon embedding");
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(icon);
    res.set("FileDescription", "Vidly");
    res.set("ProductName", "Vidly");
    res.set("LegalCopyright", "MIT License");
    if let Err(e) = res.compile() {
        // Never fail the build just because the resource compiler is unavailable.
        println!("cargo:warning=failed to embed Windows resources: {e}");
    }
}
