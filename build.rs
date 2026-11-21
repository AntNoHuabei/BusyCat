fn main() {
    println!("cargo:rerun-if-changed=assets/branding/busycat.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/branding/busycat.ico");
        resource
            .set("ProductName", "BusyCat")
            .set("FileDescription", "BusyCat Desktop Companion")
            .set("LegalCopyright", "BusyCat contributors");
        resource.compile().expect("compile BusyCat Windows icon");
    }
}
