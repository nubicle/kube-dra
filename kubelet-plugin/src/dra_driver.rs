use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;

use k8s_openapi::api::resource::v1 as resourceapi;

/// `DraDriver` is the trait that needs to be implemented by a DRA driver to
/// use the `KubeletPlugin`. The `KubeletPlugin` then implements the gRPC
/// interface expected by the kubelet by wrapping the `DraDriver` implementation.
#[async_trait]
pub trait DraDriver: Send + Sync + 'static {
    /// `prepare_resource_claims` is called to prepare all resources allocated
    /// for the given `ResourceClaim`s. This is used to implement
    /// the gRPC `node_prepare_resources` call.
    ///
    /// It gets called with the complete list of claims handled by this DRA driver
    /// that are needed by some pod. In contrast to the gRPC call, the helper has
    /// already retrieved the actual [`resourceapi::ResourceClaim`] objects.
    ///
    /// This call must be idempotent because the kubelet might have to ask
    /// for preparation multiple times, for example if it gets restarted.
    ///
    /// It is possible to create the CDI spec files which define the CDI devices
    /// on-the-fly in [`DraDriver::prepare_resource_claims`].
    /// [`DraDriver::unprepare_resource_claims`] then can
    /// remove them. Container runtimes may cache CDI specs but must reload
    /// files in case of a cache miss. To avoid false cache hits, the unique
    /// name in the CDI device ID should not be reused. A DRA driver can use
    /// the claim UID for it.
    async fn prepare_resource_claims(
        &self,
        claims: Vec<resourceapi::ResourceClaim>,
    ) -> anyhow::Result<HashMap<Uid, PrepareResult>>;

    /// `unprepare_resource_claims` must undo whatever work [`DraDriver::prepare_resource_claims`] did.
    ///
    /// At the time when this gets called, the original `ResourceClaim`s may have
    /// been deleted already. They also don't get cached by the kubelet. Therefore
    /// parameters for each [`resourceapi::ResourceClaim`] are only the UID, namespace and name.
    /// It is the responsibility of the DRA driver to cache whatever additional
    /// information it might need about prepared resources.
    ///
    /// The DRA driver cannot assume that the matching [`DraDriver::prepare_resource_claims`]
    /// call was handled by the same process.
    ///
    /// This call must be idempotent because the kubelet might have to ask
    /// for un-preparation multiple times, for example if it gets restarted.
    /// Therefore it is not an error if this gets called for a `ResourceClaim`
    /// which is not currently prepared.
    ///
    /// The conventions for returning one overall error and several per-ResourceClaim
    /// errors are the same as in [`DraDriver::prepare_resource_claims`]. In particular, all claims
    /// must have an entry in the response, even if that entry is nil.
    async fn unprepare_resource_claims(
        &self,
        claims: Vec<NamespacedObject>,
    ) -> anyhow::Result<HashMap<Uid, anyhow::Result<()>>>;

    /// `handle_error` gets called for errors encountered in the background,
    /// for example while publishing ResourceSlices.
    ///
    /// This is a mandatory method because drivers should check for errors
    /// which won't get resolved by retrying and then fail or change the
    /// slices that they are trying to publish.
    async fn handle_error(&self, err: Error, msg: &str);
}

/// `Uid` is a type that holds unique ID values, including UUIDs.  Because we
/// don't ONLY use UUIDs, this is an alias to string.  Being a type captures
/// intent and helps make sure that UIDs and names do not get conflated.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Uid(pub String);

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `PrepareResult` contains the result of preparing one particular [`resourceapi::ResourceClaim`].
///
/// `Err`, describes a problem that occurred while preparing
/// the [`resourceapi::ResourceClaim`]. The devices are then ignored and the kubelet will
/// try to prepare the ResourceClaim again later.
///
/// Devices contains the IDs of CDI devices associated with specific requests
/// in a [`resourceapi::ResourceClaim`]. Those IDs will be passed on to the container runtime
/// by the kubelet.
///
/// The empty vector is also valid.
pub type PrepareResult = anyhow::Result<Vec<Device>>;

/// `NamespacedObject` comprises a resource `name` with a mandatory `namespace`
/// and optional `Uid`. It gets rendered as `<namespace>/<name>:[<uid>]`
/// (text output) or as an object (JSON output).
#[derive(Clone)]
pub struct NamespacedObject {
    pub namespace: String,
    pub name: String,
    pub uid: Option<Uid>,
}

impl fmt::Display for NamespacedObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(uid) = &self.uid {
            write!(f, "{}/{}:{}", self.namespace, self.name, uid)
        } else {
            write!(f, "{}/{}", self.namespace, self.name)
        }
    }
}

/// `Device` provides the CDI device IDs for one request in a [`resourceapi::ResourceClaim`].
#[derive(Clone, Debug)]
pub struct Device {
    /// `requests` lists the names of requests or subrequests in the
    /// [`resourceapi::ResourceClaim`] that this device is associated with. The subrequest
    /// name may be included here, but it is also okay to just return
    /// the request name.
    ///
    /// A DRA driver can get this string from the Request field in
    /// [`resourceapi::DeviceRequestAllocationResult`], which includes the
    /// subrequest name if there is one.
    ///
    /// If empty, the device is associated with all requests.
    pub requests: Vec<String>,

    /// `pool_name` identifies the DRA driver's pool which contains the device.
    /// Must not be empty.
    pub pool_name: String,

    /// `device_name` identifies the device inside that pool.
    /// Must not be empty.
    pub device_name: String,

    /// `cdi_device_ids` lists all CDI devices associated with this DRA device.
    /// Each ID must be of the form `<vendor ID>/<class>=<unique name>`.
    /// May be empty.
    pub cdi_device_ids: Vec<String>,
}

/// `Error` is what kube-dra reports to a driver through
/// [`DraDriver::handle_error`]: failures from the library's own background
/// work — publishing `ResourceSlice`s, managing sockets, registering with
/// the kubelet — where there is no gRPC call to return them on.
///
/// Drivers are expected to match on the variants they can act on, since
/// some failures will not resolve by retrying.
#[derive(Debug, thiserror::Error)]
pub enum Error {}
