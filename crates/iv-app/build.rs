//! Embeds the Windows icon, version resource, and application manifest (specs/004 research R2).
//! Does nothing when building for other operating systems.

fn main() {
    println!("cargo:rerun-if-changed=../../packaging/windows/ImageViewer.ico");
    println!("cargo:rerun-if-changed=../../packaging/windows/ImageViewer.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    embed_windows_resources();
}

#[cfg(windows)]
fn embed_windows_resources() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../packaging/windows/ImageViewer.ico")
        .set_manifest_file("../../packaging/windows/ImageViewer.manifest")
        .set("ProductName", "ImageViewer")
        .set("FileDescription", "ImageViewer")
        .set("CompanyName", "Ezenitac-AiService")
        .set("LegalCopyright", "GPL-3.0-only; (C) edith2580, (C) 2026 Ezenitac-AiService")
        .set("OriginalFilename", "ImageViewer.exe")
        .set("ProductVersion", env!("CARGO_PKG_VERSION"));
    if let Err(e) = res.compile() {
        // A release without its icon, version, and manifest must not ship silently.
        panic!("failed to embed Windows resources: {e}");
    }
}

#[cfg(not(windows))]
fn embed_windows_resources() {
    // Cross-compiling to Windows from another host is not supported for release builds:
    // the resource compiler is only available on Windows (release.yml builds on windows-latest).
    println!("cargo:warning=Windows resources not embedded: build on a Windows host for releases");
}
