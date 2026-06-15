fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app_icon.ico");
        res.set("FileDescription", "MoHe-Ji");
        res.set("ProductName", "MoHe-Ji");
        res.set("OriginalFilename", "MoHe-Ji.exe");
        if let Err(e) = res.compile() {
            panic!("failed to embed Windows icon resource: {}", e);
        }
    }
}
