use tracing::*;

use super::*;

#[derive(Debug, Default, TransitionTo)]
#[transition_to(Self, FakeRunning)]
pub struct Registered {}

#[async_trait::async_trait]
impl State<PodState> for Registered {
    #[instrument(
        level = "info",
        skip(self, _provider_state, pod),
        fields(pod_name)
    )]
    async fn next(
        self: Box<Self>,
        _provider_state: SharedState<ProviderState>,
        pod_state: &mut PodState,
        pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        let pod = pod.latest();

        tracing::Span::current().record("pod_name", &pod.name());

        if pod_state.waited < 5 {
            info!("Only waited {} times, wait again", pod_state.waited);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            pod_state.waited += 1;
            return Transition::next(self, Self::default());
        }

        return Transition::next(self, FakeRunning::default());
    }

    async fn status(&self, _pod_state: &mut PodState, _pmeod: &Pod) -> anyhow::Result<PodStatus> {
        Ok(make_status(Phase::Pending, "Initializing"))
    }

}
