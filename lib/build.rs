use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let include_dir = Path::new("./proto").to_owned();

    let files = [
        include_dir.join("server.proto"),
        include_dir.join("player.proto"),
        // include_dir.join("queue.proto"),
        // include_dir.join("stream.proto"),
    ];

    // it seems like neither tonic_prost_build nor prost currently emit reurn instructions
    println!("cargo::rerun-if-changed={}", include_dir.display());

    // We currently need to define every single file due to https://github.com/tokio-rs/prost/issues/469
    tonic_prost_build::configure().compile_protos(&files, &[include_dir])?;

    Ok(())
}
