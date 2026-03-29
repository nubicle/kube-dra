use std::path::{self, PathBuf};

use anyhow::anyhow;

/// KUBELET_PLUGINS_DIR is the default directory for [PluginDataDirectoryPath].
const KUBELET_PLUGINS_DIR: &str = "/var/lib/kubelet/plugins";

/// KUBELET_REGISTRY_DIR is the default for [RegistrarDirectoryPath]
const KUBELET_REGISTRY_DIR: &str = "/var/lib/kubelet/plugins_registry";

const DEFAULT_GRPC_VERBOSITY: i8 = 6;

pub struct KubeletPlugin {
    driver_name: String,
    grpc_verbosity: i8,
    kube_client: kube::Client,
    node_name: String,
    plugin_data_dir: PathBuf,
}

impl KubeletPlugin {
    pub fn builder() -> KubeletPluginBuilder {
        KubeletPluginBuilder::default()
    }

    /// Start sets up all enabled gRPC servers (by default, one for registration,
    /// one for the DRA node client) and implements them by calling a [DRAPlugin]
    /// implementation.
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
    grpc_verbosity: Option<i8>,
    kube_client: Option<kube::Client>,
    node_name: Option<String>,
    plugin_data_dir: Option<PathBuf>,
}

impl KubeletPluginBuilder {
    pub fn new() -> Self {
        KubeletPluginBuilder::default()
    }

    /// DriverName defines the driver name for the dynamic resource allocation driver.
    /// Must be set. Must be a DNS subdomain and should end with a DNS domain
    /// owned by the vendor of the driver. It should use only lower case characters.
    pub fn driver_name(&mut self, name: &str) -> &mut Self {
        self.driver_name = Some(name.into());
        self
    }

    /// Sets the verbosity for logging gRPC calls.
    /// Default is `6`, which includes gRPC calls and their responses.
    /// A negative value disables logging.
    pub fn grpc_verbosity(&mut self, level: i8) -> &mut Self {
        self.grpc_verbosity = Some(level);
        self
    }

    /// Sets the path where the DRA driver creates the `dra.sock` socket that
    /// the kubelet connects to for the DRA-specific gRPC calls.
    /// It is also used to coordinate between different Pods when using rolling
    /// updates. It must not be shared with other kubelet plugins.
    ///
    /// The default is `/var/lib/kubelet/plugins/<driver name>`. This directory
    /// does not need to be inside the kubelet data directory, as long as
    /// the kubelet can access it.
    //
    /// This path must be the same inside and outside of the driver's container.
    /// The directory must exist.
    pub fn plugin_data_dir(&mut self, dir: &path::Path) -> &mut Self {
        self.plugin_data_dir = Some(dir.to_path_buf());
        self
    }

    /// KubeClient grants the plugin access to the API server. This is needed
    /// for syncing ResourceSlice objects. It's the responsibility of the DRA driver
    /// developer to ensure that this client has permission to read, write,
    /// patch and list such objects. It also needs permission to read node objects.
    /// Ideally, a validating admission policy should be used to limit write
    /// access to ResourceSlices which belong to the node.
    pub fn kube_client(&mut self, client: kube::Client) -> &mut Self {
        self.kube_client = Some(client);
        self
    }

    /// NodeName tells the plugin on which node it is running. This is needed for
    /// syncing ResourceSlice objects.
    pub fn node_name(&mut self, name: &str) -> &mut Self {
        self.node_name = Some(name.into());
        self
    }

    pub fn build(&mut self) -> anyhow::Result<KubeletPlugin> {
        let driver_name = self
            .driver_name
            .as_ref()
            .ok_or_else(|| anyhow!("driver name is required"))?;

        let kube_client = self
            .kube_client
            .as_ref()
            .ok_or_else(|| anyhow!("kubernetes client is required"))?;

        let node_name = self
            .node_name
            .as_ref()
            .ok_or_else(|| anyhow!("node name is required"))?;

        let plugin_data_dir = if self.plugin_data_dir.is_some() {
            self.plugin_data_dir.clone().unwrap()
        } else {
            PathBuf::from(format!("{KUBELET_PLUGINS_DIR}/{}", driver_name.clone()))
        };

        Ok(KubeletPlugin {
            driver_name: driver_name.to_owned(),
            grpc_verbosity: self.grpc_verbosity.unwrap_or(DEFAULT_GRPC_VERBOSITY),
            kube_client: kube_client.to_owned(),
            node_name: node_name.to_owned(),
            plugin_data_dir: plugin_data_dir,
        })
    }
}
