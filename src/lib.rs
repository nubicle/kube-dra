//! # kube-dra
//!
//! A Rust library for building Kubernetes
//! [Dynamic Resource Allocation (DRA)](https://kubernetes.io/docs/concepts/scheduling-eviction/dynamic-resource-allocation/)
//! drivers.
//!
//! ## Status
//!
//! This crate is under active development and not yet usable.
//! See the [repository](https://github.com/nubicle/kube-dra) for progress.

// `resource.k8s.io/v1` does not exist before Kubernetes 1.34, so without this a
// consumer on an older feature gets a wall of missing-type errors
k8s_openapi::k8s_if_le_1_33! {
    compile_error!("kube-dra requires the v1_34 (or higher) feature on k8s-openapi");
}

mod error;
mod kubelet_plugin;

pub use self::error::Error;
pub use self::kubelet_plugin::*;
pub use async_trait::async_trait;

/// The `k8s-openapi` version kube-dra was built against.
///
/// The public API hands drivers types from this crate —
/// [`k8s_openapi::api::resource::v1::ResourceClaim`] among them —
/// so re-exporting it lets a driver use them without adding its own
/// dependency and risking a version mismatch. A driver binary must still enable a
/// `v1_34` (or higher) feature on `k8s-openapi` itself; that choice is not kube-dra's
/// to make.
pub use k8s_openapi;
