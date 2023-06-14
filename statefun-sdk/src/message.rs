use crate::{Serializable, TypeSpec, TypedValue};

///
#[derive(Debug)]
pub struct Message {
    typed_value: TypedValue,
}

impl Message {
    ///
    pub fn is<T>(&self, typed_spec: &TypeSpec<T>) -> bool {
        self.typed_value.typename.eq(typed_spec.typename)
    }

    ///
    pub fn get<T : Serializable<T>>(&self, typed_spec: &TypeSpec<T>) -> Result<T, String> {
        if !self.is(typed_spec) {
            return Err(format!("Incompatible types. Expected: {:?} Payload: {:?}",
                typed_spec.typename, self.typed_value.typename));
        }

        T::deserialize(
            self.typed_value.typename.to_string(),
            &self.typed_value.value,
        )
    }

    ///
    pub fn get_type(&self) -> String {
        self.typed_value.typename.to_string()
    }

    ///
    pub(crate) fn new(typed_value: TypedValue) -> Self {
        Message { typed_value }
    }
}
