use std::time::Duration;

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Expiration {
    ///
    pub expiration_type : Option<ExpirationType>,

    ///
    pub time_to_live : Duration,
}

impl Expiration {
    ///
    pub fn new(expiration_type : ExpirationType, time_to_live : Duration) -> Expiration {
        Expiration {
            expiration_type : Some(expiration_type),
            time_to_live : time_to_live,
        }
    }

    ///
    pub fn never() -> Expiration {
        Expiration {
            expiration_type: None,
            time_to_live: Duration::from_secs(0),
        }
    }
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum ExpirationType {
    /// After last read or write
    AfterInvoke = 1,

    /// After initial create or the last write
    AfterWrite = 2
}
