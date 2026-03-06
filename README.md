# kube-dra

[![Rust 1.88](https://img.shields.io/badge/MSRV-1.88-dea584.svg)](https://github.com/rust-lang/rust/releases/tag/1.88.0)

A [Rust][1] library for building Kubernetes
[Dynamic Resource Allocation (DRA)][2] drivers.

> ⚠️ This crate is under active development and not yet usable.
> Watch this repository for updates.

## What is this?

DRA went [GA in Kubernetes 1.34][3]. The [Kubernetes][4] project provides
[`k8s.io/dynamic-resource-allocation/kubeletplugin`][5]
— a Go helper library that handles all the plumbing: socket registration,
gRPC lifecycle, rolling updates, and connection monitoring.

No equivalent exists for Rust. `kube-dra` aims to fill that gap.

## Status

- [x] Proto bindings
  - [x] DRA v1 and v1beta1
  - [x] Plugin registration
- [ ] Minimal kubelet-visible plugin
  - [ ] `Endpoint` — Unix socket lifecycle
  - [ ] `GrpcServer` — non-blocking tonic gRPC server
  - [ ] `RegistrationServer`
  - [ ] `NodeRegistrar`
- [ ] Public API surface
  - [ ] `DraPlugin` trait, `PrepareResult`, `Device`, `NamespacedObject`
  - [ ] Error types
- [ ] Full plugin lifecycle
  - [ ] `KubeletPluginBuilder` — configuration, start, stop, and rolling update support
- [ ] DRA handlers
  - [ ] `NodePrepareResources` and `NodeUnprepareResources`
- [ ] `ResourceSlice` publishing
  - [ ] `PublishResources` and `ResourceSlice` controller
- [ ] End-to-end example on a [kind][6] cluster

## License

Apache 2.0 licensed. See [LICENSE](./LICENSE) for details.

[1]: https://rust-lang.org/
[2]: https://kubernetes.io/docs/concepts/scheduling-eviction/dynamic-resource-allocation/
[3]: https://kubernetes.io/blog/2025/09/01/kubernetes-v1-34-dra-updates/
[4]: https://kubernetes.io
[5]: https://github.com/kubernetes/dynamic-resource-allocation/tree/master/kubeletplugin
[6]: https://kind.sigs.k8s.io/
