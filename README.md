# kube-dra

[![Rust 1.88](https://img.shields.io/badge/MSRV-1.88-dea584.svg)](https://github.com/rust-lang/rust/releases/tag/1.88.0)

A [Rust][1] library for building Kubernetes
[Dynamic Resource Allocation (DRA)][2] drivers.

> This crate is under active development and not yet usable.
> Watch this repository for updates.

## What is this?

DRA went [GA in Kubernetes 1.34][3]. The [Kubernetes][4] project provides
[`k8s.io/dynamic-resource-allocation`][5] — a set of Go packages that handle
all the plumbing a DRA driver needs: kubelet plugin registration, ResourceSlice
synchronization, gRPC lifecycle, claim fetching, device metadata, rolling
updates, and API version negotiation.

No equivalent exists for Rust. `kube-dra` aims to be a drop-in replacement.

## Architecture

The workspace mirrors the Go module structure:

```
kube-dra/
  client/            Negotiates API versions with the cluster
  resourceslice/     Watches and reconciles ResourceSlice objects
  kubeletplugin/     Kubelet plugin registration, gRPC servers, device preparation
```

A DRA driver depends on `kubeletplugin`, implements the `DraPlugin`
trait, and the library handles everything else.

## Out of scope (driver responsibility)

- Device discovery and enumeration
- Checkpoint persistence across restarts
- CDI spec writing for device access (distinct from metadata CDI)
- Opaque config decoding (driver-specific types)
- Health check server

## License

Apache 2.0 licensed. See [LICENSE](./LICENSE) for details.

[1]: https://rust-lang.org/
[2]: https://kubernetes.io/docs/concepts/scheduling-eviction/dynamic-resource-allocation/
[3]: https://kubernetes.io/blog/2025/09/01/kubernetes-v1-34-dra-updates/
[4]: https://kubernetes.io
[5]: https://github.com/kubernetes/dynamic-resource-allocation/tree/master/kubeletplugin