use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::unix::escalate_protocol::EscalateAction;
use crate::unix::escalation_policy::EscalationPolicy;
use crate::unix::stopwatch::Stopwatch;
use crate::unix::escalate_server::EscalationPolicyFactory;
use helios_utils_absolute_path::AbsolutePathBuf;
use helios_execpolicy::Policy;

#[async_trait]
#[allow(dead_code)]
pub trait ShellActionProvider: Send + Sync {
    async fn determine_action(
        &self,
        file: &AbsolutePathBuf,
        argv: &[String],
        workdir: &AbsolutePathBuf,
        stopwatch: &Stopwatch,
    ) -> anyhow::Result<EscalateAction>;
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ShellPolicyFactory {
    provider: Arc<dyn ShellActionProvider>,
}

#[allow(dead_code)]
impl ShellPolicyFactory {
    pub fn new<P>(provider: P) -> Self
    where
        P: ShellActionProvider + 'static,
    {
        Self {
            provider: Arc::new(provider),
        }
    }

    pub fn with_provider(provider: Arc<dyn ShellActionProvider>) -> Self {
        Self { provider }
    }
}

#[allow(dead_code)]
pub struct ShellEscalationPolicy {
    provider: Arc<dyn ShellActionProvider>,
    stopwatch: Stopwatch,
}

#[async_trait]
impl EscalationPolicy for ShellEscalationPolicy {
    async fn determine_action(
        &self,
        file: &AbsolutePathBuf,
        argv: &[String],
        workdir: &AbsolutePathBuf,
    ) -> anyhow::Result<EscalateAction> {
        self.provider
            .determine_action(file, argv, workdir, &self.stopwatch)
            .await
    }
}

impl EscalationPolicyFactory for ShellPolicyFactory {
    type Policy = ShellEscalationPolicy;

    fn create_policy(&self, _policy: Arc<RwLock<Policy>>, stopwatch: Stopwatch) -> Self::Policy {
        ShellEscalationPolicy {
            provider: Arc::clone(&self.provider),
            stopwatch,
        }
    }
}
