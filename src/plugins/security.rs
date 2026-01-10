use crate::common::ffi_plugin::FfiPlugin;

pub struct WireGuardPlugin {
    plugin: FfiPlugin,
}

impl WireGuardPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(WireGuardPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn create_tunnel(&self, interface: &str, config: &str) -> Result<(), String> {
        self.plugin
            .call_two_string_int_fn(b"wg_create_tunnel", interface, config)?;
        Ok(())
    }

    pub fn connect_peer(&self, peer_key: &str, endpoint: &str) -> Result<(), String> {
        self.plugin
            .call_two_string_int_fn(b"wg_connect_peer", peer_key, endpoint)?;
        Ok(())
    }

    pub fn get_status(&self, interface: &str) -> Result<String, String> {
        self.plugin.call_string_fn(b"wg_get_status", interface)
    }
}

pub struct AsciinemaPlugin {
    plugin: FfiPlugin,
}

impl AsciinemaPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(AsciinemaPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn start_recording(&self, session_name: &str) -> Result<i32, String> {
        self.plugin
            .call_string_int_fn(b"asciinema_start_recording", session_name)
    }

    pub fn stop_recording(&self, session_id: i32) -> Result<String, String> {
        self.plugin
            .call_string_fn(b"asciinema_stop_recording", &session_id.to_string())
    }

    pub fn play_recording(&self, file_path: &str) -> Result<(), String> {
        self.plugin
            .call_string_int_fn(b"asciinema_play_recording", file_path)?;
        Ok(())
    }
}

pub struct TorPlugin {
    plugin: FfiPlugin,
}

impl TorPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(TorPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn start_tor(&self, config: &str) -> Result<(), String> {
        self.plugin.call_string_int_fn(b"tor_start", config)?;
        Ok(())
    }

    pub fn create_hidden_service(&self, port: u16, target: &str) -> Result<String, String> {
        self.plugin.call_string_fn(
            b"tor_create_hidden_service",
            &format!("{}:{}", port, target),
        )
    }

    pub fn get_circuits(&self) -> Result<String, String> {
        self.plugin.call_string_fn(b"tor_get_circuits", "")
    }
}
