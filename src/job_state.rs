//! Execution state and external effect are independent. Cancellation does not
//! imply rollback; loss of a response does not make an operation fresh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Prepared,
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    Interrupted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    NotAttempted,
    Unknown,
    Confirmed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JobState {
    pub status: Status,
    pub effect: Effect,
}
impl Default for JobState {
    fn default() -> Self {
        Self {
            status: Status::Prepared,
            effect: Effect::NotAttempted,
        }
    }
}
impl JobState {
    pub fn terminal(&self) -> bool {
        matches!(
            self.status,
            Status::Succeeded | Status::Failed | Status::Cancelled | Status::Interrupted
        )
    }
    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.status != Status::Prepared {
            return Err("operation already started or finished; inspect its receipt");
        }
        self.status = Status::Queued;
        Ok(())
    }
    pub fn admitted(&mut self) -> Result<(), &'static str> {
        if self.status != Status::Queued {
            return Err("admission requires a queued operation");
        }
        self.status = Status::Running;
        Ok(())
    }
    pub fn cancel(&mut self) {
        match self.status {
            Status::Prepared => self.status = Status::Cancelled,
            Status::Queued | Status::Running => self.status = Status::Cancelling,
            _ => {}
        }
    }
    pub fn begin_publish(&mut self) -> Result<(), &'static str> {
        if self.status != Status::Running || self.effect != Effect::NotAttempted {
            return Err("publication requires a running operation without a prior attempt");
        }
        self.effect = Effect::Unknown;
        Ok(())
    }
    pub fn confirm(&mut self) -> Result<(), &'static str> {
        if self.effect != Effect::Unknown {
            return Err("no pending publication to confirm");
        }
        self.effect = Effect::Confirmed;
        self.status = Status::Succeeded;
        Ok(())
    }
    pub fn fail(&mut self) {
        if !self.terminal() {
            self.status = if self.status == Status::Cancelling {
                Status::Cancelled
            } else {
                Status::Failed
            };
        }
    }
    pub fn owner_lost(&mut self) {
        if !self.terminal() {
            self.status = Status::Interrupted;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_before_publication_has_no_effect() {
        let mut s = JobState::default();
        s.start().unwrap();
        s.cancel();
        assert!(s.begin_publish().is_err());
        s.fail();
        assert_eq!(
            s,
            JobState {
                status: Status::Cancelled,
                effect: Effect::NotAttempted
            }
        );
    }
    #[test]
    fn cancellation_during_publication_cannot_promise_rollback() {
        let mut s = JobState::default();
        s.start().unwrap();
        s.admitted().unwrap();
        s.begin_publish().unwrap();
        s.cancel();
        s.fail();
        assert_eq!(s.effect, Effect::Unknown);
        assert!(s.start().is_err());
        s.confirm().unwrap();
        assert_eq!(s.effect, Effect::Confirmed);
    }
    #[test]
    fn disconnect_does_not_reset_intent() {
        let mut s = JobState::default();
        s.start().unwrap();
        s.admitted().unwrap();
        s.begin_publish().unwrap();
        s.owner_lost();
        assert_eq!(s.effect, Effect::Unknown);
        assert!(s.start().is_err());
    }
    #[test]
    fn queue_requires_admission_and_cancellation_cannot_regress() {
        let mut state = JobState::default();
        state.start().unwrap();
        assert_eq!(state.status, Status::Queued);
        assert!(state.begin_publish().is_err());
        state.admitted().unwrap();
        assert_eq!(state.status, Status::Running);
        assert!(state.admitted().is_err());
        let mut queued = JobState::default();
        queued.start().unwrap();
        queued.cancel();
        assert_eq!(queued.status, Status::Cancelling);
        assert!(queued.admitted().is_err());
        queued.fail();
        assert_eq!(queued.status, Status::Cancelled);
        assert_eq!(queued.effect, Effect::NotAttempted);
    }
}
