fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("../logo.ico");
        resource.set("FileDescription", "EyeForge");
        resource.set("ProductName", "EyeForge");
        resource.set("OriginalFilename", "EyeForge.exe");
        resource
            .compile()
            .expect("failed to embed Windows application icon");
    }
}
