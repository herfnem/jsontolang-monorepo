pub mod lua_plugin;

use crate::schema::Document;
use anyhow::{Result, bail};
use lua_plugin::LuaPlugin;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub trait LanguagePlugin {
    fn key(&self) -> &str;
    fn render(&self, document: &Document) -> Result<String>;
}

const DEFAULT_TYPESCRIPT: &str = include_str!("../../plugins/typescript.lua");
const DEFAULT_RUST: &str = include_str!("../../plugins/rust.lua");
const DEFAULT_GO: &str = include_str!("../../plugins/go.lua");

pub fn lookup_by_key(key: &str) -> Result<Box<dyn LanguagePlugin>> {
    let result = discover();

    for warning in &result.warnings {
        eprintln!("warning: {warning}");
    }

    let mut plugins = result.plugins;

    if let Some(index) = plugins.iter().position(|plugin| plugin.key() == key) {
        return Ok(Box::new(plugins.swap_remove(index)));
    }

    let mut available: Vec<&str> = plugins.iter().map(|plugin| plugin.key()).collect();
    available.sort();
    bail!(
        "unknown language `{key}`, available: {}",
        available.join(", ")
    )
}

struct DiscoverResult {
    plugins: Vec<LuaPlugin>,
    warnings: Vec<String>,
}

fn discover() -> DiscoverResult {
    match resolve_plugin_dir() {
        Some(dir) => {
            let result = discover_dir(&dir);
            if result.plugins.is_empty() {
                fall_back_to_embedded(result)
            } else {
                result
            }
        }
        None => discover_embedded(),
    }
}

/// Falls back to the embedded default plugins when a configured plugin
/// directory produced none, while preserving any warnings that directory
/// scan produced (e.g. a custom plugin that failed to load) so they still
/// reach the user instead of being silently discarded.
fn fall_back_to_embedded(dir_result: DiscoverResult) -> DiscoverResult {
    let mut embedded = discover_embedded();
    let mut warnings = dir_result.warnings;
    warnings.append(&mut embedded.warnings);
    embedded.warnings = warnings;
    embedded
}

fn discover_embedded() -> DiscoverResult {
    let sources = [
        ("embedded:typescript.lua", DEFAULT_TYPESCRIPT),
        ("embedded:rust.lua", DEFAULT_RUST),
        ("embedded:go.lua", DEFAULT_GO),
    ];

    let mut plugins = Vec::new();
    let mut warnings = Vec::new();

    for (name, source) in sources {
        match LuaPlugin::load(source, name) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => warnings.push(format!("{name}: {error:#}")),
        }
    }

    DiscoverResult { plugins, warnings }
}

fn discover_dir(dir: &Path) -> DiscoverResult {
    let mut plugins = Vec::new();
    let mut warnings = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return DiscoverResult { plugins, warnings },
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("lua"))
        .collect();
    paths.sort();

    for path in paths {
        let name = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(source) => match LuaPlugin::load(&source, &name) {
                Ok(plugin) => plugins.push(plugin),
                Err(error) => warnings.push(format!("{name}: {error:#}")),
            },
            Err(error) => warnings.push(format!("{name}: failed to read file: {error}")),
        }
    }

    DiscoverResult { plugins, warnings }
}

fn resolve_plugin_dir_with(xdg_config_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    if let Some(xdg) = xdg_config_home.filter(|xdg| !xdg.is_empty()) {
        return Some(Path::new(xdg).join("jsontolang").join("plugins"));
    }

    let home = home?;
    Some(
        Path::new(home)
            .join(".config")
            .join("jsontolang")
            .join("plugins"),
    )
}

fn resolve_plugin_dir() -> Option<PathBuf> {
    resolve_plugin_dir_with(
        env::var("XDG_CONFIG_HOME").ok().as_deref(),
        env::var("HOME").ok().as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolve_plugin_dir_prefers_xdg_config_home() {
        let resolved = resolve_plugin_dir_with(Some("/xdg"), Some("/home/neko"));
        assert_eq!(resolved, Some(PathBuf::from("/xdg/jsontolang/plugins")));
    }

    #[test]
    fn resolve_plugin_dir_falls_back_to_home_when_xdg_unset() {
        let resolved = resolve_plugin_dir_with(None, Some("/home/neko"));
        assert_eq!(
            resolved,
            Some(PathBuf::from("/home/neko/.config/jsontolang/plugins"))
        );
    }

    #[test]
    fn resolve_plugin_dir_falls_back_to_home_when_xdg_empty() {
        let resolved = resolve_plugin_dir_with(Some(""), Some("/home/neko"));
        assert_eq!(
            resolved,
            Some(PathBuf::from("/home/neko/.config/jsontolang/plugins"))
        );
    }

    #[test]
    fn resolve_plugin_dir_is_none_when_nothing_set() {
        assert_eq!(resolve_plugin_dir_with(None, None), None);
    }

    #[test]
    fn discover_dir_skips_malformed_scripts_and_keeps_valid_ones() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("good.lua"),
            r#"return { key = "good", render = function(document) return "ok" end }"#,
        )
        .unwrap();
        fs::write(dir.path().join("bad.lua"), "this is not lua(").unwrap();

        let result = discover_dir(dir.path());

        assert_eq!(result.plugins.len(), 1);
        assert_eq!(result.plugins[0].key(), "good");
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("bad.lua"));
    }

    #[test]
    fn discover_dir_on_missing_directory_returns_empty_without_panicking() {
        let result = discover_dir(Path::new("/definitely/missing/plugin/dir"));
        assert!(result.plugins.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn discover_falls_back_to_embedded_defaults_when_dir_is_empty() {
        let dir = tempdir().unwrap();
        let result = discover_dir(dir.path());
        assert!(result.plugins.is_empty());

        let embedded = discover_embedded();
        let mut keys: Vec<&str> = embedded.plugins.iter().map(|p| p.key()).collect();
        keys.sort();
        assert_eq!(keys, vec!["go", "rust", "typescript"]);
        assert!(embedded.warnings.is_empty());
    }

    #[test]
    fn fall_back_to_embedded_preserves_dir_warnings_when_all_scripts_are_malformed() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("bad.lua"), "this is not lua(").unwrap();

        let dir_result = discover_dir(dir.path());
        assert!(dir_result.plugins.is_empty());
        assert_eq!(dir_result.warnings.len(), 1);
        assert!(dir_result.warnings[0].contains("bad.lua"));

        let result = fall_back_to_embedded(dir_result);

        // Embedded defaults are still used as the plugin set...
        let mut keys: Vec<&str> = result.plugins.iter().map(|p| p.key()).collect();
        keys.sort();
        assert_eq!(keys, vec!["go", "rust", "typescript"]);

        // ...but the directory's warning must not be lost.
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("bad.lua"));
    }

    #[test]
    fn lookup_by_key_finds_an_embedded_default_plugin() {
        let plugin = lookup_by_key("typescript").unwrap();
        assert_eq!(plugin.key(), "typescript");
    }

    #[test]
    fn lookup_by_key_reports_available_plugins_on_miss() {
        // `Box<dyn LanguagePlugin>` isn't `Debug`, so `unwrap_err()` (which
        // requires `T: Debug` on the `Ok` side) doesn't type-check here;
        // match it out instead.
        let message = match lookup_by_key("cobol") {
            Err(error) => error.to_string(),
            Ok(_) => panic!("expected lookup_by_key(\"cobol\") to fail"),
        };
        assert!(message.contains("unknown language `cobol`"));
        assert!(message.contains("go"));
        assert!(message.contains("rust"));
        assert!(message.contains("typescript"));
    }
}
