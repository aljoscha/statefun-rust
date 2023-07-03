use crate::{Serializable, TypeName, TypedValue};

/// Contains a message as received by a statefun function
#[derive(Debug)]
pub struct Message {
    typed_value: TypedValue,
}

impl Message {
    /// Check whether the received message is of the specified type.
    pub fn is<T: TypeName>(&self) -> bool {
        self.typed_value.typename.eq(T::get_typename())
    }

    /// Attempt to deserialize the message to the provided type. If the typename of the message
    /// does not match the provided type, or if deserialization fails, it will return an error.
    pub fn get<T: Serializable<T> + TypeName>(&self) -> Result<T, String> {
        if !self.is::<T>() {
            return Err(format!(
                "Incompatible types. Expected: {:?} Payload: {:?}",
                T::get_typename(),
                self.typed_value.typename
            ));
        }

        T::deserialize(
            self.typed_value.typename.to_string(),
            &self.typed_value.value,
        )
    }

    /// Get the underyling type name of this message
    pub fn get_type(&self) -> String {
        self.typed_value.typename.to_string()
    }

    ///
    pub(crate) fn new(typed_value: TypedValue) -> Self {
        Message { typed_value }
    }
}
