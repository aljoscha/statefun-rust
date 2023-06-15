use crate::{Serializable, TypedValue, GetTypename};

///
#[derive(Debug)]
pub struct Message {
    typed_value: TypedValue,
}

impl Message {
    ///
    pub fn is<T : GetTypename>(&self) -> bool {
        self.typed_value.typename.eq(T::get_typename())
    }

    ///
    pub fn get<T : Serializable<T> + GetTypename>(&self) -> Result<T, String> {
        if !self.is::<T>() {
            return Err(format!("Incompatible types. Expected: {:?} Payload: {:?}",
                T::get_typename(), self.typed_value.typename));
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
