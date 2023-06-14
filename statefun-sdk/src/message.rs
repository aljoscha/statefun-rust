use crate::{Serializable, TypedValue};

/// todo: rename this
#[derive(Debug)]
pub struct Message {
    typed_value: TypedValue,
}

impl Message {
    ///
    pub fn get<T: Serializable<T>>(&self) -> Option<T> {
        match T::deserialize(
            self.typed_value.typename.to_string(),
            &self.typed_value.value,
        ) {
            Ok(result) => Some(result),
            Err(_error) => None,  // todo: log errors
        }
    }

    ///
    pub (crate) fn new(typed_value: TypedValue) -> Self {
        Message { typed_value }
    }
}
