use std::path::{self, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;

use super::dra_server::DraServer;
use super::registration::RegistrationServer;
use crate::endpoint::Endpoint;
use crate::v1_34::dra::v1 as drav1;
use crate::v1_34::dra::v1beta1 as drav1beta1;
use crate::v1_34::plugin_registration::v1 as regv1;

/// KUBELET_PLUGINS_DIR is the default directory for [PluginDataDirectoryPath].
const KUBELET_PLUGINS_DIR: &str = "/var/lib/kubelet/plugins";

/// KUBELET_REGISTRY_DIR is the default for [RegistrarDirectoryPath]
const KUBELET_REGISTRY_DIR: &str = "/var/lib/kubelet/plugins_registry";

const DEFAULT_GRPC_VERBOSITY: i8 = 6;

pub struct KubeletPlugin {
    dra_server: Arc<DraServer>,
    reg_server: RegistrationServer,
    cancel_token: Option<CancellationToken>,
    handles: Vec<tokio::task::JoinHandle<anyhow::Result<()>>>,
}

impl KubeletPlugin {
    pub fn builder() -> KubeletPluginBuilder {
        KubeletPluginBuilder::default()
    }

    /// Start sets up all enabled gRPC servers (by default, one for registration,
    /// one for the DRA node client) and implements them by calling a [DRAPlugin]
    /// implementation.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("binding listeners");

        let dra_listener = self.dra_server.endpoint.listen().await?;
        let reg_listener = self.reg_server.endpoint.listen().await?;

        tracing::info!("binding complete, spawning servers");

        let token = CancellationToken::new();
        self.cancel_token = Some(token.clone());

        let dra_handle = tokio::spawn(start_plugin_server(
            self.dra_server.clone(),
            dra_listener,
            token.clone(),
        ));
        let reg_handle = tokio::spawn(start_registration_server(
            self.reg_server.clone(),
            reg_listener,
            token.clone(),
        ));

        self.handles = vec![dra_handle, reg_handle];

        Ok(())
    }

    pub async fn stop(self) -> anyhow::Result<()> {
        if let Some(token) = self.cancel_token {
            token.cancel();
        }

        tracing::info!("awaiting servers");
        for handle in self.handles {
            handle.await??;
        }

        tokio::fs::remove_file(self.dra_server.endpoint.path())
            .await
            .ok();

        tokio::fs::remove_file(self.reg_server.endpoint.path())
            .await
            .ok();

        Ok(())
    }
}

async fn start_plugin_server(
    server: Arc<DraServer>,
    listener: UnixListener,
    token: CancellationToken,
) -> anyhow::Result<()> {
    tonic::transport::Server::builder()
        .add_service(drav1::dra_plugin_server::DraPluginServer::new(
            server.clone(),
        ))
        .add_service(drav1beta1::dra_plugin_server::DraPluginServer::new(
            server.clone(),
        ))
        .serve_with_incoming_shutdown(UnixListenerStream::new(listener), token.cancelled())
        .await?;

    Ok(())
}

async fn start_registration_server(
    server: RegistrationServer,
    listener: UnixListener,
    token: CancellationToken,
) -> anyhow::Result<()> {
    tonic::transport::Server::builder()
        .add_service(regv1::registration_server::RegistrationServer::new(server))
        .serve_with_incoming_shutdown(UnixListenerStream::new(listener), token.cancelled())
        .await?;

    Ok(())
}

#[derive(Default)]
pub struct KubeletPluginBuilder {
    driver_name: Option<String>,
    grpc_verbosity: Option<i8>,
    kube_client: Option<kube::Client>,
    node_name: Option<String>,
    plugin_data_dir: Option<PathBuf>,
    plugin_registration_dir: Option<PathBuf>,
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

    /// RegistrarDirectoryPath sets the path to the directory where the kubelet
    /// expects to find registration sockets of plugins. Typically this is
    /// `/var/lib/kubelet/plugins_registry` with `/var/lib/kubelet` being the kubelet's
    /// data directory.
    ///
    /// This is also the default. Some Kubernetes clusters may use a different data directory.
    /// This path must be the same inside and outside of the driver's container.
    /// The directory must exist.
    pub fn registrar_directory_path(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.plugin_registration_dir = Some(dir.into());
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

        let plugin_data_dir = self
            .plugin_data_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(format!("{KUBELET_PLUGINS_DIR}/{}", driver_name)));

        let plugin_registration_dir = self
            .plugin_registration_dir
            .clone()
            .unwrap_or(PathBuf::from(KUBELET_REGISTRY_DIR));

        let dra_endpoint = Endpoint::new(plugin_data_dir, "dra.sock");
        Ok(KubeletPlugin {
            reg_server: RegistrationServer {
                driver_name: driver_name.to_owned(),
                endpoint: Endpoint::new(
                    plugin_registration_dir,
                    format!("{}-reg.sock", driver_name),
                ),
                dra_endpoint_path: dra_endpoint.path(),
            },
            dra_server: Arc::new(DraServer {
                driver_name: driver_name.to_owned(),
                grpc_verbosity: self.grpc_verbosity.unwrap_or(DEFAULT_GRPC_VERBOSITY),
                kube_client: kube_client.to_owned(),
                node_name: node_name.to_owned(),
                endpoint: dra_endpoint,
            }),
            cancel_token: None,
            handles: Vec::default(),
        })
    }
}
