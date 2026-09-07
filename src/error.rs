/// Failures originating inside kube-dra itself.
///
/// These are delivered to the driver through [`crate::DraDriver::handle_error`]
/// rather than returned from a call, because they arise in background tasks.
///
/// A driver learns two independent things from one value: **what failed**, by
/// matching on the variant, and **whether it can keep running**, from
/// [`Error::is_recoverable`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Publishing a `ResourceSlice` to the API server failed.
    #[error("publishing ResourceSlice failed")]
    ResourceSlicePublish(#[source] kube::Error),

    /// The gRPC server on the DRA socket stopped serving.
    #[error("DRA server failed")]
    DraServer(#[source] tonic::transport::Error),

    /// The gRPC server on the registration socket stopped serving.
    #[error("kubelet registration failed")]
    Registration(#[source] tonic::transport::Error),
}

impl Error {
    /// `is_recoverable` distinguishes recoverable errors from those which
    /// are fatal and should cause the process to exit.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Error::ResourceSlicePublish(_))
    }
}
