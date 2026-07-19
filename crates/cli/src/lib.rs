pub mod cli;
pub mod input;
pub mod plugins;

/// Re-exported from `jsontolang-core` so the rest of the CLI (and its tests)
/// keep referring to `jsontolang::schema::*` after the core/CLI split.
pub use jsontolang_core::schema;

use anyhow::Result;
use cli::Cli;
use plugins::lookup_by_key;
use schema::infer_document;
use std::io::Read;

pub fn run(cli: &Cli, stdin: &mut dyn Read) -> Result<String> {
    cli.validate()?;
    let value = input::read_json(cli, stdin)?;
    let document = infer_document(&cli.root, &value)?;
    lookup_by_key(&cli.lang)?.render(&document)
}
