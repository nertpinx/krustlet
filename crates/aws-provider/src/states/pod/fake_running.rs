use krator::*;
use tracing::*;

use super::*;

#[derive(Debug, Default, TransitionTo)]
#[transition_to(Self)]
pub struct FakeRunning {}

#[async_trait::async_trait]
impl State<PodState> for FakeRunning {
    async fn next(
        self: Box<Self>,
        _provider_state: SharedState<ProviderState>,
        _pod_state: &mut PodState,
        _pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        Transition::next(self, Self::default())
    }

    #[instrument(
        level = "info",
        skip(self, _pod_state, pod),
        fields(pod_name)
    )]
    async fn status(&self, _pod_state: &mut PodState, pod: &Pod) -> anyhow::Result<PodStatus> {
        tracing::Span::current().record("pod_name", &pod.name());

        let init_status = kubelet::container::Status::terminated("Finished successfully", false);
        let status = kubelet::container::Status::running();

        let init_statuses = pod
            .init_containers()
            .iter()
            .map(|c| init_status.to_kubernetes(c.name()))
            .collect();

        let app_statuses = pod
            .containers()
            .iter()
            .map(|c| {
                let mut s = status.to_kubernetes(c.name());

                if let Some(i) = c.image().unwrap_or(None) {
                    s.image = i.whole().to_owned();
                }
                s.ready = true;
                s.started = Some(true);
                s
            })
            .collect();


        let mut new_conds = vec![k8s_openapi::api::core::v1::PodCondition{
            status: "True".to_string(),
            type_: "Ready".to_string(),
            ..Default::default()
        }];

        let mut old_conds = vec![];

        if let Some(status) = &pod.as_kube_pod().status {
            if let Some(conditions) = &status.conditions {
                old_conds = conditions.iter().filter(|c| new_conds.iter().any(|nc| nc.type_ != c.type_)).cloned().collect();
            }
        }

        debug!("Old conditions are {old_conds:#?}");

        new_conds.append(&mut old_conds);

        Ok(StatusBuilder::new()
            .conditions(new_conds)
            .phase(Phase::Running)
            .reason("Running")
            // .message("") //"No, no, don't look!")
            .container_statuses(app_statuses)
            .init_container_statuses(init_statuses)
            .build())
    }
}
