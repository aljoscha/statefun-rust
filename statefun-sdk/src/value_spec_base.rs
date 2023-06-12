///
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ValueSpecBase {
    pub(crate) name: String,     // state name
    pub(crate) typename: String, // type typename
}

impl ValueSpecBase {
    ///
    pub(crate) fn new(name: &str, typename: &str) -> ValueSpecBase {
        ValueSpecBase {
            name: name.to_string(),
            typename: typename.to_string(),
        }
    }
}
