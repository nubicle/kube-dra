//! API version negotiation client for Kubernetes DRA resources.
//!
//! This crate is a placeholder. While targeting Kubernetes 1.34+ exclusively,
//! `resource.k8s.io/v1` is always available and `kube::Api` can be used
//! directly. This crate becomes necessary when adding backward compatibility
//! for clusters running 1.31–1.33 (`v1beta1` / `v1beta2` only).
//!
//! The Go equivalent is
//! [`k8s.io/dynamic-resource-allocation/client`](https://github.com/kubernetes/dynamic-resource-allocation/tree/master/client).
