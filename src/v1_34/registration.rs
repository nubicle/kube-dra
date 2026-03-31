use std::path::PathBuf;

use tonic::{Request, Response};

use crate::endpoint::Endpoint;
use crate::v1_34::dra::v1 as drav1;
use crate::v1_34::dra::v1beta1 as drav1beta1;
use crate::v1_34::plugin_registration::v1 as regv1;

/// DRAPlugin identifier for registered Dynamic Resource Allocation plugins.
const PLUGIN_TYPE: &str = "DRAPlugin";

// RegistrationServer implements the kubelet plugin registration gRPC service.
#[derive(Clone)]
pub(super) struct RegistrationServer {
    pub(super) driver_name: String,
    pub(super) endpoint: Endpoint,
    pub(super) dra_endpoint_path: PathBuf,
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
            supported_versions: vec![
                short_service_name(drav1::dra_plugin_server::SERVICE_NAME),
                short_service_name(drav1beta1::dra_plugin_server::SERVICE_NAME),
            ],
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
