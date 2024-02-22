//! Pod was deleted.

use tracing::*;

use super::*;

#[derive(Debug, Default)]
pub struct Terminated {}

#[async_trait::async_trait]
impl State<PodState> for Terminated {
    async fn next(
        self: Box<Self>,
        _provider_state: SharedState<ProviderState>,
        _pod_state: &mut PodState,
        pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        let pod = pod.latest();

        info!("Stopping pod {}", pod.name());

        Transition::Complete(Ok(()))
    }

    async fn status(&self, _pod_state: &mut PodState, _pod: &Pod) -> anyhow::Result<PodStatus> {
        Ok(make_status(Phase::Succeeded, "Terminated"))
    }
}
