fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/protos")
        .compile(
            &[
                "proto/google/cloud/speech/v1/cloud_speech.proto",
                "proto/google/api/annotations.proto", // Gerekli diğer proto dosyaları
            ],
            &["proto"],
        )?;
    Ok(())
}
