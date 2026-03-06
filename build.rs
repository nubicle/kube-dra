fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "v1_34") {
        tonic_prost_build::configure()
            .build_server(true)
            .build_client(false)
            .compile_protos(
                &[
                    "proto/v1_34/dra/v1/api.proto",
                    "proto/v1_34/dra/v1beta1/api.proto",
                    "proto/v1_34/plugin_registration/v1/api.proto",
                ],
                &["proto/v1_34", "proto/vendor"],
            )?;

        return Ok(());
    }

    Ok(())
}
