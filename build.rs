use bindgen;

use std::env;
use std::path::PathBuf;

fn main()
{
    println!("cargo:rerun-if-changed=tipc_include/tipcc.h");

    let bindings = bindgen::Builder::default()
    .header("tipc_include/tipcc.h")
    .generate()
    .expect("Unable to generate bindings!");

    let out_path= PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("binding.rs"))
    .expect("Unable to write bindings!");

    println!("cargo:rerun-if-changed=tipc_include/libtipc.c");
    cc::Build::new()
    .file("tipc_include/libtipc.c")
    .compile("libtipc");
}
