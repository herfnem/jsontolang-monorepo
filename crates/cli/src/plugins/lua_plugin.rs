use crate::plugins::LanguagePlugin;
use crate::schema::Document;
use anyhow::{Result, bail};
use mlua::{Function, Lua, LuaOptions, LuaSerdeExt, StdLib, Value};

pub struct LuaPlugin {
    lua: Lua,
    key: String,
    render_fn: Function,
}

impl LuaPlugin {
    pub fn load(source: &str, source_name: &str) -> Result<Self> {
        // mlua::Error is !Send/!Sync unless the `error-send` feature is enabled, so
        // anyhow's `.context()`/`with_context()` (which require `E: std::error::Error
        // + Send + Sync + 'static`) can't be used directly on `mlua::Result`. Instead
        // we `map_err` and format the error into an owned `String` up front, which is
        // Send + Sync and satisfies `anyhow::anyhow!`.
        let lua = Lua::new_with(
            StdLib::STRING | StdLib::TABLE | StdLib::MATH,
            LuaOptions::default(),
        )
        .map_err(|error| anyhow::anyhow!("failed to initialize sandboxed Lua runtime: {error}"))?;

        // mlua unconditionally loads Lua's base library (`_G`) regardless of the
        // `StdLib` flags above, so `os`/`io`/`package`/`debug`/`coroutine` are
        // correctly excluded but `dofile`/`loadfile`/`load` — base-lib globals that
        // can read (and `dofile` can execute) arbitrary files off disk — remain
        // present. Nil them out here, before the plugin's chunk is loaded, so
        // neither a plugin's top-level script code nor its `render` function can
        // reach the filesystem through them.
        let globals = lua.globals();
        for name in ["dofile", "loadfile", "load"] {
            globals.set(name, Value::Nil).map_err(|error| {
                anyhow::anyhow!("failed to disable base-lib global `{name}`: {error}")
            })?;
        }

        let table: mlua::Table =
            lua.load(source)
                .set_name(source_name)
                .eval()
                .map_err(|error| {
                    anyhow::anyhow!("plugin `{source_name}`: failed to load Lua source: {error}")
                })?;

        let key: String = table.get("key").map_err(|error| {
            anyhow::anyhow!("plugin `{source_name}`: missing string `key` field: {error}")
        })?;

        let render_fn: Function = table.get("render").map_err(|error| {
            anyhow::anyhow!("plugin `{source_name}`: missing `render` function field: {error}")
        })?;

        Ok(LuaPlugin {
            lua,
            key,
            render_fn,
        })
    }
}

impl LanguagePlugin for LuaPlugin {
    fn key(&self) -> &str {
        &self.key
    }

    fn render(&self, document: &Document) -> Result<String> {
        let value = self.lua.to_value(document).map_err(|error| {
            anyhow::anyhow!(
                "plugin `{}`: failed to convert document to Lua value: {error}",
                self.key
            )
        })?;

        let result: Value = self
            .render_fn
            .call(value)
            .map_err(|error| anyhow::anyhow!("plugin `{}`: render() failed: {error}", self.key))?;

        match result {
            Value::String(s) => {
                let text = s.to_str().map_err(|error| {
                    anyhow::anyhow!("plugin returned a non-UTF-8 string: {error}")
                })?;
                Ok(text.to_string())
            }
            other => bail!(
                "plugin `{}`: render() must return a string, got {}",
                self.key,
                other.type_name()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::LanguagePlugin;
    use crate::schema::infer_document;
    use serde_json::json;

    #[test]
    fn loads_and_renders_a_minimal_plugin() {
        let plugin = LuaPlugin::load(
            r#"
            return {
                key = "minimal",
                render = function(document)
                    return "root=" .. document.root_name
                end,
            }
            "#,
            "inline:minimal",
        )
        .unwrap();

        assert_eq!(plugin.key(), "minimal");

        let document = infer_document("Root", &json!({"name": "Neko"})).unwrap();
        let output = plugin.render(&document).unwrap();
        assert_eq!(output, "root=Root");
    }

    #[test]
    fn sandboxed_runtime_has_no_os_or_io_globals() {
        let plugin = LuaPlugin::load(
            r#"
            return {
                key = "probe",
                render = function(document)
                    if os == nil and io == nil then
                        return "sandboxed"
                    end
                    return "leaky"
                end,
            }
            "#,
            "inline:probe",
        )
        .unwrap();

        let document = infer_document("Root", &json!({})).unwrap();
        let output = plugin.render(&document).unwrap();
        assert_eq!(output, "sandboxed");
    }

    #[test]
    fn sandboxed_runtime_has_no_base_lib_file_loaders() {
        let plugin = LuaPlugin::load(
            r#"
            return {
                key = "probe",
                render = function(document)
                    if dofile == nil and loadfile == nil and load == nil then
                        return "sandboxed"
                    end
                    return "leaky"
                end,
            }
            "#,
            "inline:probe-fileloaders",
        )
        .unwrap();

        let document = infer_document("Root", &json!({})).unwrap();
        let output = plugin.render(&document).unwrap();
        assert_eq!(output, "sandboxed");
    }

    #[test]
    fn rejects_source_missing_render_function() {
        let result = LuaPlugin::load(r#"return { key = "broken" }"#, "inline:broken");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_syntactically_invalid_source() {
        let result = LuaPlugin::load("this is not lua(", "inline:invalid");
        assert!(result.is_err());
    }
}
