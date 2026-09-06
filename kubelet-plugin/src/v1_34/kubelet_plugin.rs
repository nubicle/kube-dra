use std::path::{self, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use tokio::net::UnixListener;
use tokio::task::JoinSet;
use tokio_stream::wrappers::UnixListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::{service::Routes, transport};

use super::dra_server::DraServer;
use super::registration::RegistrationServer;
use crate::dra_driver::DraDriver;
use crate::endpoint::Endpoint;
use crate::v1_34::dra::v1 as drav1;
use crate::v1_34::dra::v1beta1 as drav1beta1;
use crate::v1_34::plugin_registration::v1 as regv1;

/// KUBELET_PLUGINS_DIR is the default directory for `PluginDataDirectoryPath`.
const KUBELET_PLUGINS_DIR: &str = "/var/lib/kubelet/plugins";

/// KUBELET_REGISTRY_DIR is the default for `RegistrarDirectoryPath`.
const KUBELET_REGISTRY_DIR: &str = "/var/lib/kubelet/plugins_registry";

/// DEFAULT_GRPC_VERBOSITY logs each gRPC call and its response.
/// A negative value disables logging.
const DEFAULT_GRPC_VERBOSITY: i8 = 6;

/// `KubeletPlugin` is the node-local component the kubelet talks to: it owns the
/// registration and DRA gRPC servers and their sockets, and serves kubelet's calls
/// by delegating to the [`DraDriver`] handed to [`KubeletPlugin::start`].
pub struct KubeletPlugin {
    driver_name: String,
    grpc_verbosity: i8,
    kube_client: kube::Client,
    node_name: String,
    node_v1: bool,
    node_v1beta1: bool,
    endpoint: Endpoint,
    reg_server: RegistrationServer,
    cancel_token: Option<CancellationToken>,
    handles: JoinSet<()>,
}

impl KubeletPlugin {
    pub fn builder() -> KubeletPluginBuilder {
        KubeletPluginBuilder::default()
    }

    /// Start sets up all enabled gRPC servers (by default, one for registration,
    /// one for the DRA node client) and implements them by calling a [DraDriver]
    /// implementation.
    pub async fn start(&mut self, driver: Arc<dyn DraDriver>) -> anyhow::Result<()> {
        if self.cancel_token.is_some() {
            return Err(anyhow!("plugin already started"));
        }

        let dra_listener = self.endpoint.listen().await?;
        let reg_listener = self.reg_server.endpoint.listen().await?;
        let token = CancellationToken::new();
        self.cancel_token = Some(token.clone());

        let dra_server = Arc::new(DraServer {
            driver_name: self.driver_name.clone(),
            kube_client: self.kube_client.clone(),
            node_name: self.node_name.clone(),
            driver: driver.clone(),
        });

        let mut dra_routes = Routes::builder();
        if self.node_v1 {
            dra_routes.add_service(drav1::dra_plugin_server::DraPluginServer::new(
                dra_server.clone(),
            ));
        }

        if self.node_v1beta1 {
            dra_routes.add_service(drav1beta1::dra_plugin_server::DraPluginServer::new(
                dra_server.clone(),
            ));
        }

        let dc = Arc::clone(&driver);
        self.handles.spawn(start_grpc_server(
            self.grpc_verbosity,
            dra_listener,
            token.clone(),
            dra_routes.routes(),
            move |e| async move {
                dc.handle_error(crate::Error::DraServer(e)).await;
            },
        ));

        let reg_routes = Routes::default().add_service(
            regv1::registration_server::RegistrationServer::new(self.reg_server.clone()),
        );

        self.handles.spawn(start_grpc_server(
            self.grpc_verbosity,
            reg_listener,
            token.clone(),
            reg_routes,
            move |e| async move {
                driver.handle_error(crate::Error::Registration(e)).await;
            },
        ));

        Ok(())
    }

    pub async fn stop(self) -> anyhow::Result<()> {
        if let Some(token) = self.cancel_token {
            token.cancel();
        }

        let mut handles = self.handles;
        while let Some(res) = handles.join_next().await {
            if let Err(e) = res {
                tracing::error!(%e, "server task did not shut down cleanly");
            }
        }

        let rm_dra = remove_socket(&self.endpoint.path()).await;
        let rm_reg = remove_socket(&self.reg_server.endpoint.path()).await;
        rm_dra?;
        rm_reg?;

        Ok(())
    }
}

async fn remove_socket(path: &path::Path) -> anyhow::Result<()> {
    match tokio::fs::remove_file(path).await {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err.into()),
        _ => Ok(()),
    }
}

async fn start_grpc_server<F, Fut>(
    #[allow(unused_variables)] grpc_verbosity: i8,
    listener: UnixListener,
    token: CancellationToken,
    routes: Routes,
    on_error: F,
) where
    F: FnOnce(transport::Error) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    let res = transport::Server::builder()
        .add_routes(routes)
        .serve_with_incoming_shutdown(UnixListenerStream::new(listener), token.cancelled())
        .await;

    if let Err(e) = res {
        on_error(e).await;
    }
}

/// Builds a [`KubeletPlugin`]. `driver_name`, `kube_client` and `node_name` are
/// required; the socket directories and gRPC verbosity fall back to the kubelet's
/// conventional defaults. Obtain one with [`KubeletPlugin::builder`].
#[derive(Default)]
pub struct KubeletPluginBuilder {
    driver_name: Option<String>,
    grpc_verbosity: Option<i8>,
    kube_client: Option<kube::Client>,
    node_name: Option<String>,
    node_v1: Option<bool>,
    node_v1beta1: Option<bool>,
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

    /// `node_v1` explicitly chooses whether the DRA gRPC API v1
    /// gets enabled. True by default.
    ///
    /// This is used in Kubernetes for end-to-end testing. The default should
    /// be fine for DRA drivers.
    pub fn node_v1(&mut self, enabled: bool) -> &mut Self {
        self.node_v1 = Some(enabled);
        self
    }

    /// `node_v1beta1` explicitly chooses whether the DRA gRPC API v1beta1
    /// gets enabled. True by default.
    ///
    /// This is used in Kubernetes for end-to-end testing. The default should
    /// be fine for DRA drivers.
    pub fn node_v1beta1(&mut self, enabled: bool) -> &mut Self {
        self.node_v1beta1 = Some(enabled);
        self
    }

    pub fn build(&mut self) -> anyhow::Result<KubeletPlugin> {
        let node_v1 = self.node_v1.unwrap_or(true);
        let node_v1beta1 = self.node_v1beta1.unwrap_or(true);

        let supported_versions = supported_versions(node_v1, node_v1beta1);
        if supported_versions.is_empty() {
            return Err(anyhow!(
                "no supported DRA gRPC API is implemented and enabled"
            ));
        }

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
        let reg_server = RegistrationServer {
            driver_name: driver_name.to_owned(),
            endpoint: Endpoint::new(plugin_registration_dir, format!("{}-reg.sock", driver_name)),
            dra_endpoint_path: dra_endpoint.path(),
            supported_versions,
        };

        Ok(KubeletPlugin {
            driver_name: driver_name.to_owned(),
            grpc_verbosity: self.grpc_verbosity.unwrap_or(DEFAULT_GRPC_VERBOSITY),
            kube_client: kube_client.to_owned(),
            node_name: node_name.to_owned(),
            endpoint: dra_endpoint,
            cancel_token: None,
            handles: JoinSet::default(),
            reg_server,
            node_v1,
            node_v1beta1,
        })
    }
}

fn supported_versions(node_v1: bool, node_v1beta1: bool) -> Vec<String> {
    let mut versions = Vec::new();
    if node_v1 {
        versions.push(short_service_name(drav1::dra_plugin_server::SERVICE_NAME));
    }

    if node_v1beta1 {
        versions.push(short_service_name(
            drav1beta1::dra_plugin_server::SERVICE_NAME,
        ));
    }

    versions
}

fn short_service_name(svc: &str) -> String {
    let mut parts = svc.rsplitn(3, '.');
    let last = parts.next().unwrap_or("");
    let second_last = parts.next().unwrap_or("");
    format!("{second_last}.{last}")
}

#[cfg(test)]
mod tests {
    use crate::v1_34::dra::v1 as drav1;
    use crate::v1_34::dra::v1beta1 as drav1beta1;

    #[test]
    fn generates_correct_short_name() {
        let v1_svc = super::short_service_name(drav1::dra_plugin_server::SERVICE_NAME);
        assert_eq!(v1_svc, String::from("v1.DRAPlugin"));

        let v1beta1_svc = super::short_service_name(drav1beta1::dra_plugin_server::SERVICE_NAME);
        assert_eq!(v1beta1_svc, String::from("v1beta1.DRAPlugin"));
    }
}
