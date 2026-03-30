use std::sync::Arc;

use crate::endpoint::Endpoint;
use crate::v1_34::dra::v1 as drav1;
use crate::v1_34::dra::v1beta1 as drav1beta1;

// DraServer implements the DraPlugin gRPC service.
pub(super) struct DraServer {
    pub(super) driver_name: String,
    pub(super) grpc_verbosity: i8,
    pub(super) kube_client: kube::Client,
    pub(super) node_name: String,
    pub(super) endpoint: Endpoint,
}

#[tonic::async_trait]
impl drav1::dra_plugin_server::DraPlugin for Arc<DraServer> {
    async fn node_prepare_resources(
        &self,
        _request: tonic::Request<drav1::NodePrepareResourcesRequest>,
    ) -> Result<tonic::Response<drav1::NodePrepareResourcesResponse>, tonic::Status> {
        todo!()
    }

    async fn node_unprepare_resources(
        &self,
        _request: tonic::Request<drav1::NodeUnprepareResourcesRequest>,
    ) -> Result<tonic::Response<drav1::NodeUnprepareResourcesResponse>, tonic::Status> {
        todo!()
    }
}

#[tonic::async_trait]
impl drav1beta1::dra_plugin_server::DraPlugin for Arc<DraServer> {
    async fn node_prepare_resources(
        &self,
        _request: tonic::Request<drav1beta1::NodePrepareResourcesRequest>,
    ) -> Result<tonic::Response<drav1beta1::NodePrepareResourcesResponse>, tonic::Status> {
        todo!()
    }

    async fn node_unprepare_resources(
        &self,
        _request: tonic::Request<drav1beta1::NodeUnprepareResourcesRequest>,
    ) -> Result<tonic::Response<drav1beta1::NodeUnprepareResourcesResponse>, tonic::Status> {
        todo!()
    }
}
