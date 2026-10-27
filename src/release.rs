//! A release identity is checked before operational work, never adopted from a
//! stale request. Discovery and receipt inspection remain independently usable.
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseId(String);

impl ReleaseId {
    pub fn parse(value: &str) -> Result<Self, ReleaseError> {
        if value.is_empty()
            || value.len() > 256
            || !value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c))
        {
            return Err(ReleaseError::InvalidIdentity);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn require(&self, expected: &str) -> Result<(), ReleaseError> {
        let expected = Self::parse(expected)?;
        if *self == expected {
            Ok(())
        } else {
            Err(ReleaseError::Mismatch)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseError {
    InvalidIdentity,
    Mismatch,
    InvalidPayload,
}
impl fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidIdentity => "release identity is missing or invalid",
            Self::Mismatch => {
                "release changed; inspect current operation receipts before issuing new intent"
            }
            Self::InvalidPayload => {
                "release payload requires an explicit existing Node executable path"
            }
        })
    }
}
impl std::error::Error for ReleaseError {}

/// Development qualification may explicitly select an installed executable.
/// Installed releases must supply their verified private payload paths instead.
#[derive(Clone, Debug)]
pub struct AdapterPayload {
    pub node: PathBuf,
    pub diagram: Option<PathBuf>,
    pub design: Option<crate::design::DesignPayload>,
    pub openpencil: Option<crate::openpencil::OpenPencilPayload>,
    pub video: Option<crate::video::VideoPayload>,
}
impl AdapterPayload {
    pub fn explicit(node: &Path) -> Result<Self, ReleaseError> {
        if !node.is_absolute() || !node.is_file() {
            return Err(ReleaseError::InvalidPayload);
        }
        Ok(Self {
            diagram: None,
            design: None,
            openpencil: None,
            video: None,
            node: node
                .canonicalize()
                .map_err(|_| ReleaseError::InvalidPayload)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_identity_is_not_adopted() {
        let current = ReleaseId::parse("0.0.1:fixture-b").unwrap();
        assert_eq!(
            current.require("0.0.1:fixture-a"),
            Err(ReleaseError::Mismatch)
        );
        assert_eq!(current.as_str(), "0.0.1:fixture-b");
        assert!(current.require("0.0.1:fixture-b").is_ok());
    }
    #[test]
    fn identity_is_bounded_and_explicit() {
        for value in ["", "release\nother", "../bad/path", "a b", "ç"] {
            assert!(ReleaseId::parse(value).is_err());
        }
        assert!(ReleaseId::parse(&"a".repeat(257)).is_err());
    }
    #[test]
    fn payload_never_uses_path_lookup() {
        assert!(AdapterPayload::explicit(Path::new("node")).is_err());
    }
}
