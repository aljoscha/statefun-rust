use std::fmt::{Display, Formatter};
use statefun_proto::request_reply::Address as ProtoAddress;
use crate::FunctionType;

/// The unique identity of an individual stateful function.
///
/// This comprises the function's `FunctionType` and an unique identifier within the
/// type. The function's type denotes the class of function to invoke, while the unique identifier
/// addresses the invocation to a specific function instance.
///
/// This must be used when sending messages to stateful functions as part of the function
/// [Effects](Effects).
#[derive(Debug, PartialEq)]
pub struct Address {
    /// `FunctionType` of the stateful function that this `Address` refers to.
    pub function_type: FunctionType,

    /// Unique id of the stateful function that this `Address` refers to.
    pub id: String,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Address {}/{}", self.function_type, self.id)
    }
}

impl Address {
    /// Creates a new `Address` from the given `FunctionType` and id.
    pub fn new(function_type: FunctionType, id: &str) -> Self {
        Address {
            function_type,
            id: id.to_owned(),
        }
    }

    /// Converts the Protobuf `Address` into an `Address`. We don't implement `From`/`Into` for this
    /// because we want to keep it out of the public API.
    pub fn from_proto(proto_address: &ProtoAddress) -> Self {
        Address {
            function_type: FunctionType::new(
                proto_address.get_namespace(),
                proto_address.get_field_type(),
            ),
            id: proto_address.get_id().to_owned(),
        }
    }

    /// Converts this `Address` into a Protobuf `Address`. We don't implement `From`/`Into` for this
    /// because we want to keep it out of the public API.
    pub fn into_proto(self) -> ProtoAddress {
        let mut result = ProtoAddress::new();
        result.set_namespace(self.function_type.get_namespace());
        result.set_field_type(self.function_type.get_name());
        result.set_id(self.id);
        result
    }
}
