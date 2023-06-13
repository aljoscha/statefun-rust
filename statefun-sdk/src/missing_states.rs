use crate::ValueSpecBase;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// blabla
#[derive(Error, PartialEq, Eq, Hash, Debug)]
pub struct MissingStates {
    pub(crate) states: Vec<ValueSpecBase>,
}

impl MissingStates {
    /// blabla
    pub fn new(states: Vec<ValueSpecBase>) -> MissingStates {
        MissingStates { states }
    }
}

impl Display for MissingStates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MissingStates {:?}", self.states)
    }
}
