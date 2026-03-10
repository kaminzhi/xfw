use std::env;
use std::path::PathBuf;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub config_path: PathBuf,
}

impl CliOptions {
    pub fn parse() -> Result<Self> {
        let mut args = env::args().skip(1);
        let mut config_path: Option<PathBuf> = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--config" | "-c" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--config requires a path"))?;
                    config_path = Some(PathBuf::from(value));
                }
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }

        let path = config_path.unwrap_or_else(|| PathBuf::from("lua/widgets/status_bar.lua"));
        Ok(Self { config_path: path })
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub entrypoint: PathBuf,
}

impl From<CliOptions> for RuntimeConfig {
    fn from(value: CliOptions) -> Self {
        Self {
            entrypoint: value.config_path,
        }
    }
}
