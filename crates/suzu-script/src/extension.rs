use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommandSpec {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    commands: HashSet<String>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_command(&mut self, spec: CustomCommandSpec) {
        self.commands.insert(spec.name);
    }

    pub fn register_command_name(&mut self, name: impl Into<String>) {
        self.register_command(CustomCommandSpec { name: name.into() });
    }

    pub fn contains_command(&self, name: &str) -> bool {
        self.commands.contains(name)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(feature = "lua")]
impl ExtensionRegistry {
    pub fn register_lua_command_list(&mut self, source: &str) -> mlua::Result<()> {
        let lua = mlua::Lua::new();
        let commands: mlua::Table = lua.load(source).eval()?;
        for command in commands.sequence_values::<String>() {
            self.register_command_name(command?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_tracks_custom_commands() {
        let mut registry = ExtensionRegistry::new();
        registry.register_command_name("shakeui");

        assert!(registry.contains_command("shakeui"));
        assert_eq!(registry.len(), 1);
    }

    #[cfg(feature = "lua")]
    #[test]
    fn lua_command_list_registers_commands() {
        let mut registry = ExtensionRegistry::new();

        registry
            .register_lua_command_list(r#"return { "shakeui", "unlock_gallery" }"#)
            .unwrap();

        assert!(registry.contains_command("shakeui"));
        assert!(registry.contains_command("unlock_gallery"));
    }
}
