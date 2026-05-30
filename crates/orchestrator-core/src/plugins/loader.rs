use libloading::Library;
use std::collections::HashMap;
use std::path::Path;

/// Plugin registry for dynamic libraries (stub for V1).
pub struct PluginRegistry {
    plugins: HashMap<String, Library>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Loads a plugin from a dynamic library path.
    pub fn load_plugin(&mut self, path: impl AsRef<Path>) -> anyhow::Result<String> {
        let path = path.as_ref();
        let lib = unsafe { Library::new(path) }?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.plugins.insert(name.clone(), lib);
        Ok(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}
