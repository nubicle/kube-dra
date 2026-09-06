use std::path::PathBuf;

use tonic::{Request, Response};

use crate::endpoint::Endpoint;
use crate::v1_34::plugin_registration::v1 as regv1;

/// DRAPlugin identifier for registered Dynamic Resource Allocation plugins.
const PLUGIN_TYPE: &str = "DRAPlugin";

// RegistrationServer implements the kubelet plugin registration gRPC service.
#[derive(Clone)]
pub(super) struct RegistrationServer {
    pub(super) driver_name: String,
    pub(super) endpoint: Endpoint,
    pub(super) dra_endpoint_path: PathBuf,
    pub(super) supported_versions: Vec<String>,
}

#[tonic::async_trait]
impl regv1::registration_server::Registration for RegistrationServer {
    /// get_info is the RPC invoked by plugin watcher.
    async fn get_info(
        &self,
        _: Request<regv1::InfoRequest>,
    ) -> Result<Response<regv1::PluginInfo>, tonic::Status> {
        let info = regv1::PluginInfo {
            name: self.driver_name.clone(),
            endpoint: self.dra_endpoint_path.to_string_lossy().to_string(),
            r#type: String::from(PLUGIN_TYPE),
            supported_versions: self.supported_versions.clone(),
        };

        Ok(Response::new(info))
    }

    /// notify_registration_status is the RPC invoked by plugin watcher.
    async fn notify_registration_status(
        &self,
        status: Request<regv1::RegistrationStatus>,
    ) -> Result<Response<regv1::RegistrationStatusResponse>, tonic::Status> {
        let status = status.into_inner();

        if !status.plugin_registered {
            tracing::error!(
                driver = %self.driver_name,
                error = %status.error,
                "registration failed"
            );

            return Err(tonic::Status::internal(format!(
                "failed registration process: {}",
                status.error
            )));
        }

        tracing::info!(driver = %self.driver_name, "registration successful");
        Ok(Response::new(regv1::RegistrationStatusResponse::default()))
    }
}
