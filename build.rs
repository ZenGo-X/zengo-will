fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile(
            &["proto/beneficiary.proto", "proto/testator.proto"],
            &["proto/"],
        )?;
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir("examples/proto")
        .compile(
            &["proto/beneficiary.proto", "proto/testator.proto"],
            &["proto/"],
        )?;
    Ok(())
}
