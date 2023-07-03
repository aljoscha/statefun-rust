use std::fmt::{Display, Formatter};

/// A reference to a stateful function, consisting of a namespace and a name.
///
/// A function's type is part of a function's [Address](Address) and serves as integral part of an
/// individual function's identity.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FunctionType {
    namespace: String,
    name: String,
}

impl FunctionType {
    /// Creates a new `FunctionType` from the given namespace and name.
    pub fn new(namespace: &str, name: &str) -> FunctionType {
        FunctionType {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }

    /// Get the namespace of this function
    pub fn get_namespace(&self) -> String {
        self.namespace.to_string()
    }

    /// Get the name of this function
    pub fn get_name(&self) -> String {
        self.name.to_string()
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionType {}/{}", self.namespace, self.name)
    }
}
