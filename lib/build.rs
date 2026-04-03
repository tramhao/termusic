fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/player.proto")?;
    tonic_prost_build::compile_protos("proto/monitor.proto")?;
    Ok(())
}
