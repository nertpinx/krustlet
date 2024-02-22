mod fake_running;
mod registered;
mod terminated;

use std::collections::HashMap;

pub use fake_running::*;
pub use registered::*;
pub use terminated::*;

pub use kubelet::state::common::{BackoffSequence, GenericPodState, ThresholdTrigger};
pub use kubelet::pod::make_status;
pub use kubelet::pod::state::prelude::*;


use crate::*;


const BACKOFF_INCREASE_SECS: u64 = 1;
const BACKOFF_SECS_MAX: u64 = 10;

#[derive(Debug, Default)]
/// PodState
pub struct PodState {
    last_backoff_secs: u64,
    waited: u8,
}

#[async_trait::async_trait]
impl krator::ObjectState for PodState {
    type Manifest = Pod;
    type Status = kubelet::pod::Status;
    type SharedState = ProviderState;
    async fn async_drop(self, _provider_state: &mut ProviderState) {}
}

#[async_trait::async_trait]
impl GenericPodState for PodState {
    async fn set_env_vars(&mut self, _env_vars: HashMap<String, HashMap<String, String>>) {
        unimplemented!();
    }
    async fn set_modules(&mut self, _modules: HashMap<String, Vec<u8>>) {
        unimplemented!();
    }
    async fn set_volumes(&mut self, _volumes: HashMap<String, kubelet::volume::VolumeRef>) {
        unimplemented!();
    }
    async fn backoff(&mut self, sequence: BackoffSequence) {
        if self.last_backoff_secs < BACKOFF_SECS_MAX {
            self.last_backoff_secs += BACKOFF_INCREASE_SECS;
        }
        if self.last_backoff_secs > BACKOFF_SECS_MAX {
            self.last_backoff_secs = BACKOFF_SECS_MAX;
        }

        tracing::warn!("Backing off for sequence {}",
                       match sequence {
                           BackoffSequence::CrashLoop => "CrashLoop",
                           BackoffSequence::ImagePull => "ImagePull",
                       });
        tokio::time::sleep(tokio::time::Duration::from_secs(self.last_backoff_secs)).await;
    }
    async fn reset_backoff(&mut self, _sequence: BackoffSequence) {
        self.last_backoff_secs = 0;
    }

    async fn record_error(&mut self) -> ThresholdTrigger {
        ThresholdTrigger::Triggered
    }
}
