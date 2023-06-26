use crate::ValueSpecBase;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Used to mark missing state by the function, which will be messaged back to the Flink Statefun
/// cluster. The cluster should subsequently initialize the state and call the same function again.
#[derive(Error, PartialEq, Eq, Hash, Debug)]
pub struct MissingStates {
    pub(crate) states: Vec<ValueSpecBase>,
}

impl MissingStates {
    ///
    pub fn new(states: Vec<ValueSpecBase>) -> MissingStates {
        MissingStates { states }
    }
}

impl Display for MissingStates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MissingStates {:?}", self.states)
    }
}
