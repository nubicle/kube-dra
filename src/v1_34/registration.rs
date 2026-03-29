use tonic::{Request, Response};

use super::dra;
use super::plugin_registration::v1::{self, registration_server::Registration};

/// DRAPlugin identifier for registered Dynamic Resource Allocation plugins.
const PLUGIN_TYPE: &str = "DRAPlugin";

// RegistrationServer implements the kubelet plugin registration gRPC service.
pub(super) struct RegistrationServer {
    driver_name: String,
    endpoint: String,
}

impl RegistrationServer {
    pub fn new(driver_name: &str, endpoint: &str) -> Self {
        RegistrationServer {
            driver_name: driver_name.to_string(),
            endpoint: endpoint.to_string(),
        }
    }
}

#[tonic::async_trait]
impl Registration for RegistrationServer {
    /// get_info is the RPC invoked by plugin watcher.
    async fn get_info(
        &self,
        _: Request<v1::InfoRequest>,
    ) -> Result<Response<v1::PluginInfo>, tonic::Status> {
        let info = v1::PluginInfo {
            name: self.driver_name.clone(),
            endpoint: self.endpoint.clone(),
            r#type: String::from(PLUGIN_TYPE),
            supported_versions: vec![
                dra::v1::dra_plugin_server::SERVICE_NAME.to_string(),
                dra::v1beta1::dra_plugin_server::SERVICE_NAME.to_string(),
            ],
        };

        Ok(Response::new(info))
    }

    /// notify_registration_status is the RPC invoked by plugin watcher.
    async fn notify_registration_status(
        &self,
        status: Request<v1::RegistrationStatus>,
    ) -> Result<Response<v1::RegistrationStatusResponse>, tonic::Status> {
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
        Ok(Response::new(v1::RegistrationStatusResponse::default()))
    }
}
