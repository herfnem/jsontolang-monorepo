use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "jsontolang")]
#[command(group(
    ArgGroup::new("input")
        .args(["file", "stdin", "json"])
        .required(true)
        .multiple(false)
))]
pub struct Cli {
    #[arg(long)]
    pub lang: String,

    #[arg(long, default_value = "Root")]
    pub root: String,

    #[arg(long)]
    pub file: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub stdin: bool,

    #[arg(long)]
    pub json: Option<String>,
}

impl Cli {
    pub fn validate(&self) -> anyhow::Result<()> {
        let mut count = 0;

        if self.file.is_some() {
            count += 1;
        }
        if self.stdin {
            count += 1;
        }
        if self.json.is_some() {
            count += 1;
        }

        anyhow::ensure!(
            count == 1,
            "provide exactly one of --file, --stdin, or --json"
        );

        Ok(())
    }
}
