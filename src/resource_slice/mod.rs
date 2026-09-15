use std::collections::HashMap;

use k8s_openapi::api::resource::v1 as resourceapi;

/// `DriverResources` is a complete description of all resources synchronized by the controller.
pub struct DriverResources {
    /// Each driver may manage different resource pools.
    ///
    /// The key in the map is the pool name. Pools are
    /// sorted first so that pools with devices which
    /// have binding conditions are tried last, then by name.
    /// So the name can also be used to indicate preference
    /// when a driver publishes more than one pool.
    pub pools: HashMap<String, Pool>,
}

/// `Pool` is the collection of devices belonging to the same pool.
pub struct Pool {
    /// `generation` can be left at zero. It gets bumped up automatically
    /// by the controller.
    pub generation: i64,

    /// `slices` is a list of all ResourceSlices that the driver
    /// wants to publish for this pool. The driver must ensure
    /// that each resulting slice is valid. See the API
    /// definition for details, in particular the limit on
    /// the number of devices.
    ///
    /// ResourceSlices start with a name prefix based on their
    /// index in this list. This has two important consequences:
    /// 1. Priority: The allocator sorts ResourceSlices
    ///    lexicographically by name and uses a first-fit strategy.
    ///    Since the index is at the beginning of the name, the order
    ///    in this list determines the allocation priority. Driver
    ///    authors can influence priority by putting preferred slices
    ///    first. Likewise, within a slice the preferred devices should
    ///    be listed first.
    /// 2. Migration: When upgrading from a driver version that used
    ///    randomly generated names without index at the beginning,
    ///    existing ResourceSlices will be deleted and recreated with
    ///    names starting with the new prefix.
    ///
    /// The name prefix follows the scheme:
    /// `[index encoded as base16 string]-[driver name]-[owner name (if present)]-`
    ///
    /// If slices are not valid, then the controller will
    /// log errors produced by the API server.
    ///
    /// Drivers should publish at least one slice for each
    /// pool that they normally manage, even if that slice
    /// is empty. "Empty pool" is different from "no pool"
    /// because it shows that the driver is up-and-running
    /// and simply doesn't have any devices.
    pub slices: Vec<Slice>,
}

/// `Slice` is turned into one ResourceSlice by the controller.
pub struct Slice {
    /// `devices` lists all devices which are part of the slice.
    pub devices: Vec<resourceapi::Device>,
}
