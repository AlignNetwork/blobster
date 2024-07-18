fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OUT_DIR: {}", std::env::var("OUT_DIR").unwrap_or_else(|_| "Not set".to_string()));
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/exex.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    println!("cargo:rerun-if-changed=proto/exex.proto");
    Ok(())
}
