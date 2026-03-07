use std::path::{self, PathBuf};

pub struct KubeletPlugin {
    driver_name: Option<String>,
    plugin_dir: Option<PathBuf>,
    registrar_dir: Option<PathBuf>,
}

impl KubeletPlugin {
    pub fn builder() -> KubeletPluginBuilder {
        KubeletPluginBuilder::default()
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn stop(self) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Default)]
pub struct KubeletPluginBuilder {
    driver_name: Option<String>,
    plugin_dir: Option<PathBuf>,
    registrar_dir: Option<PathBuf>,
}

impl KubeletPluginBuilder {
    pub fn new() -> Self {
        KubeletPluginBuilder::default()
    }

    pub fn driver_name(&mut self, name: &str) -> &mut Self {
        self.driver_name = Some(name.into());

        self
    }

    pub fn plugin_dir(&mut self, dir: &path::Path) -> &mut Self {
        self.plugin_dir = Some(dir.to_path_buf());

        self
    }

    pub fn registrar_dir(&mut self, dir: &path::Path) -> &mut Self {
        self.registrar_dir = Some(dir.to_path_buf());

        self
    }

    pub fn build(&mut self) -> KubeletPlugin {
        KubeletPlugin {
            driver_name: self.driver_name.to_owned(),
            plugin_dir: self.plugin_dir.to_owned(),
            registrar_dir: self.registrar_dir.to_owned(),
        }
    }
}
