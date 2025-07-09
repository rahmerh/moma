use crate::bundles::bundle::Bundle;

pub mod nexus;

trait Source {
    async fn setup_source() -> anyhow::Result<()>;
    fn bundle_info_for(bundle_uid: u64) -> anyhow::Result<Bundle>;
}
