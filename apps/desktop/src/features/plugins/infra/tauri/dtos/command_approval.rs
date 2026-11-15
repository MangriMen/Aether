use aether_core::features::plugins::{
    CommandApprovalAnswer, CommandJar, OnceOnlyReason, PendingCommandApproval,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// A command a plugin wants to run, waiting for the user.
#[derive(Debug, Clone, Serialize, Type)]
pub struct PendingCommandApprovalDto {
    pub request_id: String,
    pub plugin_id: String,
    /// The Java launcher the command starts.
    pub java: String,
    pub current_dir: String,
    /// The whole argument list, the JVM options and the jar included.
    pub arguments: Vec<String>,
    pub jar: Option<CommandJarDto>,
    /// Why "always" is not offered; `None` when it is.
    pub once_only: Option<OnceOnlyReasonDto>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct CommandJarDto {
    pub plugin_path: String,
    pub sha256: String,
    /// A number, not `u64`: specta refuses to export `u64`, and a jar over 2^53 bytes is refused
    /// long before.
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OnceOnlyReasonDto {
    NotJarLaunch,
    RepeatedJarFlag,
    ArgFile,
    UnlistedOption { option: String },
    TargetNotInCache,
    JarHasClassPath,
    JarUnreadable,
    JarTooLarge,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CommandAnswerDto {
    Once,
    Always,
    Deny,
}

impl From<PendingCommandApproval> for PendingCommandApprovalDto {
    fn from(pending: PendingCommandApproval) -> Self {
        let request = pending.request;
        Self {
            request_id: request.id.to_string(),
            plugin_id: request.plugin_id.into(),
            java: request.java,
            current_dir: request.current_dir,
            arguments: request.arguments,
            jar: request.jar.map(Into::into),
            once_only: request.always.err().map(Into::into),
            expires_at: pending.expires_at,
        }
    }
}

impl From<CommandJar> for CommandJarDto {
    #[allow(clippy::cast_precision_loss)]
    fn from(jar: CommandJar) -> Self {
        Self {
            plugin_path: jar.plugin_path,
            sha256: jar.sha256,
            size: jar.size as f64,
        }
    }
}

impl From<OnceOnlyReason> for OnceOnlyReasonDto {
    fn from(reason: OnceOnlyReason) -> Self {
        match reason {
            OnceOnlyReason::NotJarLaunch => Self::NotJarLaunch,
            OnceOnlyReason::RepeatedJarFlag => Self::RepeatedJarFlag,
            OnceOnlyReason::ArgFile => Self::ArgFile,
            OnceOnlyReason::UnlistedOption { option } => Self::UnlistedOption { option },
            OnceOnlyReason::TargetNotInCache => Self::TargetNotInCache,
            OnceOnlyReason::JarHasClassPath => Self::JarHasClassPath,
            OnceOnlyReason::JarUnreadable => Self::JarUnreadable,
            OnceOnlyReason::JarTooLarge => Self::JarTooLarge,
        }
    }
}

impl From<CommandAnswerDto> for CommandApprovalAnswer {
    fn from(answer: CommandAnswerDto) -> Self {
        match answer {
            CommandAnswerDto::Once => Self::Once,
            CommandAnswerDto::Always => Self::Always,
            CommandAnswerDto::Deny => Self::Deny,
        }
    }
}
