use crate::ValueSpecBase;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// blabla
#[derive(Error, PartialEq, Eq, Hash, Debug)]
pub struct MissingStateCollection {
    pub(crate) states: Vec<ValueSpecBase>,
}

impl MissingStateCollection {
    /// blabla
    pub fn new(states: Vec<ValueSpecBase>) -> MissingStateCollection {
        MissingStateCollection { states: states }
    }
}

impl Display for MissingStateCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MissingStateCollection {:?}", self.states)
    }
}
