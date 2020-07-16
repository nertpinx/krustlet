//! `container` is a collection of utilities surrounding the Kubernetes container API.

use k8s_openapi::api::core::v1::Container as KubeContainer;
use oci_distribution::Reference;
use std::convert::TryInto;

mod handle;
mod status;

pub use handle::Handle;
pub use status::Status;

/// Specifies how the store should check for module updates
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PullPolicy {
    /// Always return the module as it currently appears in the
    /// upstream registry
    Always,
    /// Return the module as it is currently cached in the local store if
    /// present; fetch it from the upstream registry only if it it not
    /// present in the local store
    IfNotPresent,
    /// Never fetch the module from the upstream registry; if it is not
    /// available locally then return an error
    Never,
}

impl PullPolicy {
    /// Get image pull policy of container applying defaults if None from:
    /// https://kubernetes.io/docs/concepts/configuration/overview/#container-images
    pub fn parse_effective(policy: Option<&str>, image: Option<Reference>) -> anyhow::Result<Self> {
        match PullPolicy::parse(policy)? {
            Some(policy) => Ok(policy),
            None => match image {
                Some(image) => match image.tag() {
                    Some("latest") | None => Ok(PullPolicy::Always),
                    _ => Ok(PullPolicy::IfNotPresent),
                },
                None => Ok(PullPolicy::IfNotPresent),
            },
        }
    }

    /// Parses a module pull policy from a Kubernetes ImagePullPolicy string
    pub fn parse(name: Option<&str>) -> anyhow::Result<Option<Self>> {
        match name {
            None => Ok(None),
            Some(s) => Self::parse_str(s),
        }
    }

    fn parse_str(name: &str) -> anyhow::Result<Option<Self>> {
        match name {
            "Always" => Ok(Some(Self::Always)),
            "IfNotPresent" => Ok(Some(Self::IfNotPresent)),
            "Never" => Ok(Some(Self::Never)),
            other => Err(anyhow::anyhow!("unrecognized pull policy {}", other)),
        }
    }
}

/// A Kubernetes Container
///
/// This is a new type around the k8s_openapi Container definition
/// providing convenient accessor methods
#[derive(Default, Debug, Clone)]
pub struct Container(KubeContainer);

impl Container {
    /// Create new Container from KubeContainer
    pub fn new(container: &KubeContainer) -> Self {
        Container(container.clone())
    }

    /// Get arguments of container.
    pub fn args(&self) -> &Option<Vec<String>> {
        &self.0.args
    }

    /// Get command of container.
    pub fn command(&self) -> &Option<Vec<String>> {
        &self.0.command
    }

    /// Get environment of container.
    pub fn env(&self) -> &Option<Vec<k8s_openapi::api::core::v1::EnvVar>> {
        &self.0.env
    }

    /// Get environment of container.
    pub fn env_from(&self) -> &Option<Vec<k8s_openapi::api::core::v1::EnvFromSource>> {
        &self.0.env_from
    }

    /// Get image of container as `oci_distribution::Reference`.
    pub fn image(&self) -> anyhow::Result<Option<Reference>> {
        match self.0.image.as_ref() {
            Some(s) => Some(s.clone().try_into()).transpose(),
            None => Ok(None),
        }
    }

    /// Get effective pull policy of container.
    pub fn effective_pull_policy(&self) -> anyhow::Result<PullPolicy> {
        PullPolicy::parse_effective(self.0.image_pull_policy.as_deref(), self.image()?)
    }

    /// Get lifecycle of container.
    pub fn lifecycle(&self) -> Option<&k8s_openapi::api::core::v1::Lifecycle> {
        self.0.lifecycle.as_ref()
    }

    /// Get liveness probe of container.
    pub fn liveness_probe(&self) -> Option<&k8s_openapi::api::core::v1::Probe> {
        self.0.liveness_probe.as_ref()
    }

    /// Get name of container.
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// Get ports of container.
    pub fn ports(&self) -> &Option<Vec<k8s_openapi::api::core::v1::ContainerPort>> {
        &self.0.ports
    }

    /// Get readiness probe of container.
    pub fn readiness_probe(&self) -> Option<&k8s_openapi::api::core::v1::Probe> {
        self.0.readiness_probe.as_ref()
    }

    /// Get resources of container.
    pub fn resources(&self) -> Option<&k8s_openapi::api::core::v1::ResourceRequirements> {
        self.0.resources.as_ref()
    }

    /// Get security context of container.
    pub fn security_context(&self) -> Option<&k8s_openapi::api::core::v1::SecurityContext> {
        self.0.security_context.as_ref()
    }

    /// Get startup probe of container.
    pub fn startup_probe(&self) -> Option<&k8s_openapi::api::core::v1::Probe> {
        self.0.startup_probe.as_ref()
    }

    /// Get stdin flag of container.
    pub fn stdin(&self) -> Option<bool> {
        self.0.stdin
    }

    /// Get stdin_once flag of container.
    pub fn stdin_once(&self) -> Option<bool> {
        self.0.stdin_once
    }

    /// Get termination message path of container.
    pub fn termination_message_path(&self) -> Option<&String> {
        self.0.termination_message_path.as_ref()
    }

    /// Get termination message policy of container.
    pub fn termination_message_policy(&self) -> Option<&String> {
        self.0.termination_message_policy.as_ref()
    }

    /// Get tty flag of container.
    pub fn tty(&self) -> Option<bool> {
        self.0.tty
    }

    /// Get volume devices of container.
    pub fn volume_devices(&self) -> &Option<Vec<k8s_openapi::api::core::v1::VolumeDevice>> {
        &self.0.volume_devices
    }

    /// Get volume mounts of container.
    pub fn volume_mounts(&self) -> &Option<Vec<k8s_openapi::api::core::v1::VolumeMount>> {
        &self.0.volume_mounts
    }

    /// Get working directory of container.
    pub fn working_dir(&self) -> Option<&String> {
        self.0.working_dir.as_ref()
    }
}