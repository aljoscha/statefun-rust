use crate::Address;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct DelayedInvocation {
    pub address: Address,
    pub delay: Duration,
    pub cancellation_token: String,
    pub typename: String,
    pub bytes: Vec<u8>,
}

impl DelayedInvocation {
    pub fn new(
        address: Address,
        delay: Duration,
        cancellation_token: String,
        typename: String,
        bytes: Vec<u8>,
    ) -> DelayedInvocation {
        DelayedInvocation {
            address,
            delay,
            cancellation_token,
            typename,
            bytes,
        }
    }
}
