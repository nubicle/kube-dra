pub(super) mod dra {
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

pub(super) mod plugin;
mod registration;
