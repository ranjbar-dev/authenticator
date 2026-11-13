fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=assets/authenticator.ico");
    println!("cargo:rerun-if-changed=assets/windows.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let assembly_version = format!(
            "{}.{}.{}.0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH")
        );
        let manifest = include_str!("assets/windows.manifest")
            .replace("AUTHENTICATOR_VERSION", &assembly_version);
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("assets/authenticator.ico")
            .set_manifest(&manifest)
            .set_language(0x0409)
            .set("ProductName", "Authenticator")
            .set("FileDescription", "Authenticator")
            .set("OriginalFilename", "Authenticator.exe")
            .set("InternalName", "Authenticator")
            .set("CompanyName", "Authenticator contributors")
            .set(
                "LegalCopyright",
                "Copyright 2026 Authenticator contributors",
            );
        resource.compile()?;
    }
    Ok(())
}
