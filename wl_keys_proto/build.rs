fn main() -> anyhow::Result<()> {
    tonic_build::compile_protos("proto/wl_keys.proto")?;
    Ok(())
}
