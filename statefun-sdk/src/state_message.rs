use crate::{deserializer, Serializable, TypedValue};

///
#[derive(Debug)]
pub struct StateMessage {
    typed_value: TypedValue,
}

impl StateMessage {
    ///
    pub fn get<T: Serializable>(&self) -> Option<T> {
        // todo: make deserializer return Option
        Some(deserializer::<T>(
            self.typed_value.typename.to_string(),
            &self.typed_value.value,
        ))
    }

    ///
    pub fn new(typed_value: TypedValue) -> Self {
        StateMessage {
            typed_value,
        }
    }
}
