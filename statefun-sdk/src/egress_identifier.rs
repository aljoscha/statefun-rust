use std::fmt::{Display, Formatter};

/// A reference to an _egress_, consisting of a namespace and a name.
///
/// This has to be used when sending messages to an egress as part of the function
/// [Effects](Effects).
#[derive(Debug)]
pub struct EgressIdentifier {
    pub (crate) namespace: String,
    pub (crate) name: String,
}

impl EgressIdentifier {
    /// Creates a new `EgressIdentifier` from the given namespace and name.
    pub fn new(namespace: &str, name: &str) -> EgressIdentifier {
        EgressIdentifier {
            namespace: namespace.to_string(),
            name: name.to_string(),
        }
    }
}

impl Display for EgressIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EgressIdentifier {}/{}", self.namespace, self.name)
    }
}
