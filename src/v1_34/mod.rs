mod dra {
    mod v1 {
        include!(concat!(
            env!("OUT_DIR"),
            "/k8s.io.kubelet.pkg.apis.dra.v1.rs"
        ));
    }

    mod v1beta1 {
        include!(concat!(
            env!("OUT_DIR"),
            "/k8s.io.kubelet.pkg.apis.dra.v1beta1.rs"
        ));
    }
}

mod plugin_registration {
    pub(crate) mod v1 {
        include!(concat!(env!("OUT_DIR"), "/pluginregistration.rs"));
    }
}

pub(super) mod plugin;
