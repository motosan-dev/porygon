fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let descriptor_path = format!("{out_dir}/descriptors.bin");

    // Use pbjson_types for well-known google.protobuf types (Struct, Value, Timestamp)
    // so they get proper serde support.
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .extern_path(".google.protobuf.Struct", "::pbjson_types::Struct")
        .extern_path(".google.protobuf.Value", "::pbjson_types::Value")
        .extern_path(".google.protobuf.Timestamp", "::pbjson_types::Timestamp")
        .compile_protos(&["proto/a2a.proto"], &["proto/"])
        .unwrap();

    let descriptor_set = std::fs::read(&descriptor_path).unwrap();
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)
        .unwrap()
        .extern_path(".google.protobuf.Struct", "::pbjson_types::Struct")
        .extern_path(".google.protobuf.Value", "::pbjson_types::Value")
        .extern_path(".google.protobuf.Timestamp", "::pbjson_types::Timestamp")
        .build(&[".lf.a2a.v1"])
        .unwrap();
}
