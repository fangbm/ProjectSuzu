#[cfg(windows)]
fn main() {
    set_windows_icon();
}

#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
fn set_windows_icon() {
    winresource::WindowsResource::new()
        .set_icon("../../assets/branding/Suzu_icon.ico")
        .compile()
        .expect("failed to embed Project Suzu icon");
}
