pub trait DraPlugin: Send + Sync {
    async fn prepare_resource_claims(&self);
}

#[derive(Default)]
pub struct Internal;

impl DraPlugin for Internal {
    async fn prepare_resource_claims(&self) {
        // do something
    }
}
