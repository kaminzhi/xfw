use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let cli = xfw_cli::CliOptions::parse()?;
    let runtime_cfg = xfw_cli::RuntimeConfig::from(cli);
    let mut runtime = xfw_runtime::Runtime::new(runtime_cfg)?;
    runtime.run()
}
