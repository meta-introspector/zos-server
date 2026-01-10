use crate::common::ffi_plugin::FfiPlugin;

pub struct DockerPlugin {
    plugin: FfiPlugin,
}

impl DockerPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(DockerPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn create_container(&self, image: &str, name: &str) -> Result<String, String> {
        self.plugin
            .call_string_fn(b"docker_create_container", &format!("{}:{}", image, name))
    }

    pub fn start_container(&self, container_id: &str) -> Result<(), String> {
        self.plugin
            .call_string_int_fn(b"docker_start_container", container_id)?;
        Ok(())
    }

    pub fn stop_container(&self, container_id: &str) -> Result<(), String> {
        self.plugin
            .call_string_int_fn(b"docker_stop_container", container_id)?;
        Ok(())
    }

    pub fn list_containers(&self) -> Result<String, String> {
        self.plugin.call_string_fn(b"docker_list_containers", "")
    }
}

pub struct SystemdPlugin {
    plugin: FfiPlugin,
}

impl SystemdPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        Ok(SystemdPlugin {
            plugin: FfiPlugin::new(plugin_path)?,
        })
    }

    pub fn start_service(&self, service_name: &str) -> Result<(), String> {
        self.plugin
            .call_string_int_fn(b"systemd_start_service", service_name)?;
        Ok(())
    }

    pub fn stop_service(&self, service_name: &str) -> Result<(), String> {
        self.plugin
            .call_string_int_fn(b"systemd_stop_service", service_name)?;
        Ok(())
    }

    pub fn get_service_status(&self, service_name: &str) -> Result<String, String> {
        self.plugin
            .call_string_fn(b"systemd_get_status", service_name)
    }

    pub fn enable_service(&self, service_name: &str) -> Result<(), String> {
        self.plugin
            .call_string_int_fn(b"systemd_enable_service", service_name)?;
        Ok(())
    }
}
