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

pub(crate) mod driver;
pub(crate) mod plugin;
