fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    println!("cargo:warning= target is {}", target);
    if target == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
        println!("cargo:warning= icon properly loaded!");
    } else {
        println!("cargo:warning= windows not target os...");
    }
    panic!("--- FORCED CRASH TO READ LOGS ---");
}
