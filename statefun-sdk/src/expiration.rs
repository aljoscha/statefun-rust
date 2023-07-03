use std::time::Duration;

/// Specifies the expiration type and time to live for a given state
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Expiration {
    ///
    pub expiration_type: Option<ExpirationType>,

    ///
    pub time_to_live: Duration,
}

impl Expiration {
    /// Constructor
    pub fn new(expiration_type: ExpirationType, time_to_live: Duration) -> Expiration {
        Expiration {
            expiration_type: Some(expiration_type),
            time_to_live,
        }
    }

    /// Helper function to mark the state as never expiring
    pub fn never() -> Expiration {
        Expiration {
            expiration_type: None,
            time_to_live: Duration::from_secs(0),
        }
    }
}

/// Specifies the expiration time for a given state
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExpirationType {
    /// After last read or write
    AfterInvoke = 1,

    /// After initial create or the last write
    AfterWrite = 2,
}
