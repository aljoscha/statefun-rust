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
    pub fn get<T: Serializable<T>>(&self) -> Result<T, String> {
        T::deserialize(
            self.typed_value.typename.to_string(),
            &self.typed_value.value,
        )
    }

    ///
    pub fn get_typed_value(&self) -> TypedValue {
        self.typed_value.clone()
    }

    ///
    pub(crate) fn new(typed_value: TypedValue) -> Self {
        Message { typed_value }
    }
}
