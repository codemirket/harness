//! Resolve an installed session's verified, immutable release payload.
//! The caller retains its session guard through engine startup so maintenance
//! cannot change the active release in between.

use crate::{distribution::VerifiedRelease, maintenance::SessionGuard, release::AdapterPayload};
use std::path::Path;

pub fn resolve_installed_payload(
    release: &VerifiedRelease,
    state_root: &Path,
    session: &SessionGuard,
) -> Result<AdapterPayload, String> {
    if session.state_root() != state_root {
        return Err("installed payload requires this installation's session guard".into());
    }
    Ok(release.payload())
}
