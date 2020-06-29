use std::{
    env, fs,
    ops::Deref,
    path::{Path, PathBuf},
};

fn out_dir() -> PathBuf {
    Path::new(&env::var("OUT_DIR").expect("env")).join("proto")
}

fn cleanup() {
    let _ = fs::remove_dir_all(&out_dir());
}

#[allow(deprecated)]
fn compile() {
    let proto_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").expect("env")).join("src");

    eprintln!("proto_dir={:?}", proto_dir);

    let files = glob::glob(&proto_dir.join("**/*.proto").to_string_lossy())
        .expect("glob")
        .filter_map(|p| p.ok().map(|p| p.to_string_lossy().into_owned()))
        .collect::<Vec<_>>();

    let slices = files.iter().map(Deref::deref).collect::<Vec<_>>();

    eprintln!("protos: {:?}", slices);

    let out_dir = out_dir();
    fs::create_dir(&out_dir).expect("create_dir");

    protoc_rust::run(protoc_rust::Args {
        out_dir: &out_dir.to_string_lossy(),
        input: &slices,
        includes: &[&proto_dir.to_string_lossy()],
        customize: protoc_rust::Customize {
            ..Default::default()
        },
    })
    .expect("protoc");
}

fn generate_mod_rs() {
    let out_dir = out_dir();

    let mods = glob::glob(&out_dir.join("*.rs").to_string_lossy())
        .expect("glob")
        .filter_map(|p| {
            p.ok()
                .map(|p| format!("pub mod {};", p.file_stem().unwrap().to_string_lossy()))
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mod_rs = out_dir.join("mod.rs");
    fs::write(&mod_rs, format!("// @generated\n{}\n", mods)).expect("write");

    println!("cargo:rustc-env=PROTO_MOD_RS={}", mod_rs.to_string_lossy());
}

fn main() {
    cleanup();
    compile();
    generate_mod_rs();
}
