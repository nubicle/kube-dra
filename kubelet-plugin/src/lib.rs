//! Kubelet plugin framework for Kubernetes Dynamic Resource Allocation (DRA) drivers.
//!
//! This crate handles the plumbing — socket registration, gRPC lifecycle,
//! and connection monitoring — so you can focus on your driver logic.

mod dra_driver;
pub use self::dra_driver::*;

#[cfg(feature = "v1_34")]
mod v1_34;

#[cfg(feature = "v1_34")]
pub use self::v1_34::kubelet_plugin::*;

pub use async_trait::async_trait;

mod endpoint;
