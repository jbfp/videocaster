extern crate embed_resource;

fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=app.ico");
        println!("cargo:rerun-if-changed=app.rc");
        embed_resource::compile("app.rc");
    }
}
