use crate::cli::Cli;
use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, io::Read};

pub fn read_json(cli: &Cli, stdin: &mut dyn Read) -> Result<Value> {
    let raw = if let Some(path) = &cli.file {
        fs::read_to_string(path)
            .with_context(|| format!("failed to read input file `{}`", path.display()))?
    } else if cli.stdin {
        let mut buffer = String::new();
        stdin
            .read_to_string(&mut buffer)
            .context("failed to read JSON from stdin")?;
        buffer
    } else if let Some(json) = &cli.json {
        json.clone()
    } else {
        anyhow::bail!("provide exactly one of --file, --stdin, or --json");
    };

    serde_json::from_str(&raw).context("invalid JSON input")
}
