//! Model discovery and the only model execution boundary owned by Mirket.
mod assignment;
mod codex;
mod design_gate;
mod responses;
pub use assignment::{SkillReference, TaskNeed};
pub use design_gate::DesignInput;
pub(crate) mod discovery;
pub(crate) mod execution;
pub(crate) mod native;
pub mod policy;
pub(crate) mod tools;

pub use discovery::{Registry, discover};
pub use execution::{plan, spawn};
pub use policy::SpawnRequest;
