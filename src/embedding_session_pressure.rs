//! Cached installation resource observations for an already-owned warm leaf.
//! Cold load and warm dispatch always recheck; idle polling does not kill active
//! inference. Registration and process-tree sampling live with resource admission.
use crate::resource_admission::{self, Snapshot};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const REFRESH: Duration = Duration::from_millis(250);
pub(super) struct Monitor {
    root: PathBuf,
    sample: Option<Snapshot>,
    observed: Instant,
}
impl Monitor {
    pub(super) fn new(root: &Path) -> Result<Self, String> {
        Ok(Self {
            root: root.to_owned(),
            sample: None,
            observed: Instant::now(),
        })
    }
    fn refresh(&mut self) -> Result<Option<bool>, String> {
        match resource_admission::snapshot(&self.root) {
            Ok(sample) => {
                self.sample = Some(sample);
                self.observed = Instant::now();
                Ok(Some(sample.defers()))
            }
            // Contention is not a pressure observation. Dispatch must wait for
            // a fresh result; an already-idle model can wait for the next tick.
            Err(resource_admission::Error::Busy) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    pub(super) fn elevated(&mut self) -> Result<Option<bool>, String> {
        if self.sample.is_none() || self.observed.elapsed() >= REFRESH {
            self.refresh()
        } else {
            Ok(self.sample.map(|sample| sample.defers()))
        }
    }
    pub(super) fn before_dispatch(&mut self) -> Result<Option<bool>, String> {
        self.refresh()
    }
}
