//! Cooperating state users hold shared leases; recovery owns the exclusive lease.
use crate::{paths::Paths, util};
use anyhow::{Context, Result, ensure};

pub fn shared(paths: &Paths) -> Result<util::FileLock> {
    paths.ensure()?;
    let lease = util::FileLock::shared(&paths.root.join("durable.lock"))
        .context("durable state maintenance is active; retry after it completes")?;
    ensure!(
        !paths.root.join("restore-pending.json").exists(),
        "durable restore is incomplete; repeat the same backup restore command"
    );
    Ok(lease)
}

pub fn exclusive(paths: &Paths) -> Result<util::FileLock> {
    paths.ensure()?;
    util::FileLock::acquire(&paths.root.join("durable.lock"))
        .context("durable state is in use; wait for workers and state calls to finish")
}
