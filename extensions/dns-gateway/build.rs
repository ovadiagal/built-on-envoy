use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path().unwrap());

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let proto_dir = root.join("proto");
    let proto_files: Vec<PathBuf> = std::fs::read_dir(&proto_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension() == Some("proto".as_ref()))
        .collect();
    let includes = &[proto_dir.clone()];

    println!("cargo:rerun-if-changed={}", proto_dir.display());

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("proto_descriptor.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&proto_files, includes)?;

    let descriptor_set = std::fs::read(&descriptor_path)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .build(&["."])?;

    Ok(())
}
