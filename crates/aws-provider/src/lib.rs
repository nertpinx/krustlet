use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use kubelet::node::Builder;
use kubelet::plugin_watcher::PluginRegistry;
use kubelet::provider::*;
use kubelet::resources::DeviceManager;
use kubelet::store::Store;
use tokio::sync::RwLock;
use tracing::*;

use kubelet::pod::Pod;

mod states;

const LOG_DIR_NAME: &str = "aws-logs";
const VOLUME_DIR: &str = "volumes";

#[derive(Clone)]
pub struct AWSProvider {
    shared: ProviderState,
}

#[derive(Clone)]
pub struct ProviderState {
    _store: Arc<dyn Store + Sync + Send>,
    _log_path: PathBuf,
    _client: kube::Client,
    volume_path: PathBuf,
    plugin_registry: Arc<PluginRegistry>,
    device_plugin_manager: Arc<DeviceManager>,
}

impl VolumeSupport for ProviderState {
    fn volume_path(&self) -> Option<&Path> {
        Some(self.volume_path.as_ref())
    }
}

impl PluginSupport for ProviderState {
    fn plugin_registry(&self) -> Option<Arc<PluginRegistry>> {
        Some(self.plugin_registry.clone())
    }
}

impl DevicePluginSupport for ProviderState {
    fn device_plugin_manager(&self) -> Option<Arc<DeviceManager>> {
        Some(self.device_plugin_manager.clone())
    }
}

impl AWSProvider {
    /// New
    pub async fn new(
        store: Arc<dyn Store + Sync + Send>,
        config: &kubelet::config::Config,
        kubeconfig: kube::Config,
        plugin_registry: Arc<PluginRegistry>,
        device_plugin_manager: Arc<DeviceManager>,
    ) -> anyhow::Result<Self> {
        let log_path = config.data_dir.join(LOG_DIR_NAME);
        let volume_path = config.data_dir.join(VOLUME_DIR);
        tokio::fs::create_dir_all(&log_path).await?;
        tokio::fs::create_dir_all(&volume_path).await?;
        let client = kube::Client::try_from(kubeconfig)?;
        Ok(Self {
            shared: ProviderState {
                _store: store,
                _log_path: log_path,
                volume_path,
                _client: client,
                plugin_registry,
                device_plugin_manager,
            },
        })
    }
}

#[async_trait]
impl Provider for AWSProvider {
    type ProviderState = ProviderState;
    type InitialState = states::pod::Registered;
    type TerminatedState = states::pod::Terminated;
    type PodState = states::pod::PodState;

    const ARCH: &'static str = "amd64";

    async fn node(&self, builder: &mut Builder) -> anyhow::Result<()> {
        builder.set_architecture(Self::ARCH);
        builder.add_taint("NoSchedule", "kubernetes.io/arch", Self::ARCH);
        builder.add_taint("NoExecute", "kubernetes.io/arch", Self::ARCH);
        Ok(())
    }

    fn provider_state(&self) -> krator::SharedState<ProviderState> {
        Arc::new(RwLock::new(self.shared.clone()))
    }

    async fn initialize_pod_state(&self, _pod: &Pod) -> anyhow::Result<Self::PodState> {
        Ok(Default::default())
    }

    async fn logs(
        &self,
        _namespace: String,
        _pod: String,
        _container: String,
        _sender: kubelet::log::Sender,
    ) -> anyhow::Result<()> {
        warn!("Someone wants logs, oopsie!");
        todo!()
    }
}
