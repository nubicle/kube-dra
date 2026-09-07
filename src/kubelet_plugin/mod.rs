//! Kubelet plugin framework for DRA drivers.
//!
//! Handles the plumbing — socket registration, gRPC lifecycle, and connection
//! monitoring — so a driver only has to implement [`DraDriver`].

mod dra {
    pub(super) mod v1 {
        include!(concat!(
            env!("OUT_DIR"),
            "/k8s.io.kubelet.pkg.apis.dra.v1.rs"
        ));
    }

    pub(super) mod v1beta1 {
        include!(concat!(
            env!("OUT_DIR"),
            "/k8s.io.kubelet.pkg.apis.dra.v1beta1.rs"
        ));
    }
}

mod plugin_registration {
    pub(super) mod v1 {
        include!(concat!(env!("OUT_DIR"), "/pluginregistration.rs"));
    }
}

pub use self::dra_driver::*;
pub use self::plugin::*;

mod dra_driver;
mod dra_server;
mod endpoint;
mod plugin;
mod registration;
