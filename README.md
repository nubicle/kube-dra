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

## Scope

`kube-dra` targets feature parity with the Go [`kubeletplugin`][5] package.

### `client`

| Feature                                                               | Priority | Size | Status |
| --------------------------------------------------------------------- | -------- | ---- | ------ |
| Version-negotiating resource client (v1 → v1beta2 → v1beta1 fallback) | P1       | M    |        |
| ResourceSlice CRUD through negotiated API                             | P1       | S    |        |
| ResourceClaim read through negotiated API                             | P1       | S    |        |
| Watch stream with on-the-fly type conversion                          | P1       | M    |        |

### `resourceslice`

| Feature                                                                                                         | Priority | Size | Status |
| --------------------------------------------------------------------------------------------------------------- | -------- | ---- | ------ |
| Core types <br> - `DriverResources` <br> - `Pool` <br> - `Slice` <br> - `Owner`                                 | P1       | S    |        |
| Kubernetes watch loop filtered by driver name + node name                                                       | P1       | M    |        |
| Work queue keyed by pool name with rate limiting                                                                | P1       | M    |        |
| Sync delay (debounce) between informer events and reconciliation                                                | P1       | S    |        |
| Per-pool reconciliation <br> - create missing slices <br> - update changed slices <br> - delete obsolete slices | P1       | L    |        |
| Generation tracking across slices in a pool                                                                     | P1       | S    |        |
| Owner reference management (Node as owner, UID lookup)                                                          | P1       | S    |        |
| Mutation cache with TTL to avoid create/informer race                                                           | P2       | M    |        |
| Slice naming scheme (hex-encoded index prefix)                                                                  | P2       | S    |        |
| Dropped fields detection (feature gate awareness)                                                               | P3       | S    |        |
| Controller lifecycle <br> - `StartController` <br> - `Update` <br> - `Stop`                                     | P1       | M    |        |
| ResourceSlice validation before submission                                                                      | P2       | S    |        |

### `kubeletplugin`

| Feature                                                                                                         | Priority | Size | Status |
| --------------------------------------------------------------------------------------------------------------- | -------- | ---- | ------ |
| DRA v1 and v1beta1 proto bindings                                                                               | P1       | S    | ✅     |
| Plugin registration v1 proto bindings                                                                           | P1       | XS   | ✅     |
| `Endpoint` <br> - Unix socket create <br> - stale cleanup <br> - delete                                         | P1       | S    | ✅     |
| Registration gRPC server <br> - `GetInfo` <br> - `NotifyRegistrationStatus`                                     | P1       | S    | ✅     |
| DRA gRPC server — v1 + v1beta1 `DraPlugin` service (stubs)                                                      | P1       | S    | ✅     |
| `KubeletPlugin` + `KubeletPluginBuilder`                                                                        | P1       | M    | ✅     |
| `start()` — bind listeners, spawn servers                                                                       | P1       | S    | ✅     |
| `stop()` <br> - cancel token <br> - await handles <br> - remove sockets                                         | P1       | S    | ✅     |
| Double-start guard                                                                                              | P1       | XS   | ✅     |
| h2 `:authority` patch for Unix domain sockets ([nubicle/h2][6] fork)                                            | P1       | S    | ✅     |
| `DraPlugin` trait <br> - `prepare_resource_claims` <br> - `unprepare_resource_claims` <br> - `handle_error`     | P1       | S    |        |
| Public types <br> - `PrepareResult` <br> - `Device` <br> - `NamespacedObject`                                   | P1       | S    |        |
| Error types <br> - `ErrRecoverable` sentinel <br> - typed errors                                                | P1       | S    |        |
| `NodePrepareResources` handler <br> - fetch claims <br> - validate UIDs <br> - call trait <br> - build response | P1       | M    |        |
| `NodeUnprepareResources` handler — call trait, build response                                                   | P1       | S    |        |
| ResourceClaim fetching + UID validation before prepare                                                          | P1       | S    |        |
| Subrequest name stripping (`BaseRequestRef`)                                                                    | P2       | XS   |        |
| CDI device ID injection into gRPC response                                                                      | P1       | S    |        |
| `publish_resources` — lazy-start controller on first call, update on subsequent calls                           | P1       | S    |        |
| In-process mutex for prepare/unprepare serialization                                                            | P1       | XS   |        |
| POSIX flock for rolling update cross-pod serialization                                                          | P2       | S    |        |
| Unary logging interceptor (request ID, method, verbosity)                                                       | P2       | S    |        |
| Streaming logging interceptor                                                                                   | P2       | S    |        |
| Context merging (startup logger into per-call context)                                                          | P2       | S    |        |
| Metadata JSON writer (multi-version stream format)                                                              | P2       | M    |        |
| CDI spec injection for metadata bind-mount                                                                      | P2       | M    |        |
| Metadata cleanup on unprepare                                                                                   | P2       | S    |        |
| `update_request_metadata` for post-sandbox network attributes                                                   | P3       | S    |        |
| `DRAResourceHealth` gRPC service (if driver implements it)                                                      | P3       | S    |        |

### Out of scope (driver responsibility)

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
[6]: https://github.com/nubicle/h2/tree/patch-authority
