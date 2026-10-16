//! Shared host identities and installation/project paths.
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Codex,
    Claude,
}
impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl Target {
    pub(crate) fn parse(value: &str) -> anyhow::Result<Self> {
        match value {
            "codex" => Ok(Self::Codex),
            "claude" => Ok(Self::Claude),
            _ => anyhow::bail!("unknown target: {value}"),
        }
    }
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
    pub(crate) fn instructions(self) -> &'static str {
        match self {
            Self::Codex => ".codex/AGENTS.md",
            Self::Claude => ".claude/CLAUDE.md",
        }
    }
    pub(crate) fn skills(self) -> &'static str {
        match self {
            Self::Codex => ".agents/skills",
            Self::Claude => ".claude/skills",
        }
    }
    pub(crate) fn config(self) -> &'static str {
        match self {
            Self::Codex => ".codex/config.toml",
            Self::Claude => ".claude.json",
        }
    }
}
