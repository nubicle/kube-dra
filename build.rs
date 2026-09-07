fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &[
                "proto/dra/v1/api.proto",
                "proto/dra/v1beta1/api.proto",
                "proto/plugin_registration/v1/api.proto",
            ],
            &["proto", "proto/vendor"],
        )?;

    Ok(())
}
