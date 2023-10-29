fn main() {
    compile_capnp();
}

fn compile_capnp() {
    capnpc::CompilerCommand::new()
        .src_prefix("src")
        .file("src/system/system.capnp")
        .default_parent_module(vec!["system".into()])
        .run()
        .expect("compiling schema");

    println!("cargo:rerun-if-changed=src/system/system.capnp");
}
