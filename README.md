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

## Kubernetes versions

`kube-dra` targets one Kubernetes minor per release, the same way
[`k8s.io/dynamic-resource-allocation`][5] is published one module version per
Kubernetes minor — `v0.34.x` for 1.34, `v0.35.x` for 1.35.

| kube-dra | Kubernetes |
| -------- | ---------- |
| 0.1.x    | 1.34       |

The driver binary picks the API types by enabling the matching version feature on
[`k8s-openapi`][8]; `kube-dra` requires `v1_34` or higher and fails to compile below
it. Which DRA gRPC generations get served — `v1`, `v1beta1`, or both — is a separate,
runtime choice on the plugin builder.

## Architecture

A single crate, mirroring the Go module's package structure:

```
kube-dra/
  src/
    resource_slice.rs   Watches and reconciles ResourceSlice objects
    kubelet_plugin/     Kubelet plugin registration, gRPC servers, device preparation
```

A DRA driver depends on `kube-dra`, implements the [`DraDriver`][7]
trait, and the library handles everything else.

## Example

[`kube-dra-example-driver`][6] is a reference DRA driver built on `kube-dra`,
packaged with a Helm chart and a one-command kind setup.

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
[6]: https://github.com/nubicle/kube-dra-example-driver
[7]: src/kubelet_plugin/dra_driver.rs
[8]: https://docs.rs/k8s-openapi/latest/k8s_openapi/
