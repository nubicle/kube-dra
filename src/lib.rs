//! # kube-dra
//!
//! A Rust library for building Kubernetes
//! [Dynamic Resource Allocation (DRA)](https://kubernetes.io/docs/concepts/scheduling-eviction/dynamic-resource-allocation/)
//! drivers.
//!
//! This crate is the Rust equivalent of the Go
//! [`k8s.io/dynamic-resource-allocation/kubeletplugin`](https://github.com/kubernetes/dynamic-resource-allocation/tree/master/kubeletplugin)
//! helper library. It handles the plumbing — socket registration, gRPC
//! lifecycle, and connection monitoring — so you can focus on your
//! driver logic.
//!
//! ## Status
//!
//! This crate is under active development and not yet usable.
//! See the [repository](https://github.com/nubicle/kube-dra) for progress.

#[cfg(feature = "v1_34")]
pub(crate) mod dra {
    pub(crate) mod v1 {
        include!(concat!(
            env!("OUT_DIR"),
            "/k8s.io.kubelet.pkg.apis.dra.v1.rs"
        ));
    }

    pub(crate) mod v1beta1 {
        include!(concat!(
            env!("OUT_DIR"),
            "/k8s.io.kubelet.pkg.apis.dra.v1beta1.rs"
        ));
    }
}

#[cfg(feature = "v1_34")]
pub(crate) mod plugin_registration {
    pub(crate) mod v1 {
        include!(concat!(env!("OUT_DIR"), "/pluginregistration.rs"));
    }
}
