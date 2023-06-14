//! Provides [KafkaEgress](crate::io::kafka::KafkaEgress) for sending egress messages to Kafka.
//!
//! To use this, import the `KafkaEgress` trait and then use
//! [`kafka_egress()`](crate::io::kafka::KafkaEgress::kafka_egress) or
//! [`kafka_keyed_egress()`](crate::io::kafka::KafkaEgress::kafka_keyed_egress) on an
//! [Effects](crate::Effects) to send messages to Kafka.
//!
//! # Examples
//!
//! ```
//! use protobuf::well_known_types::StringValue;
//!
//! use statefun::io::kafka::KafkaEgress;
//! use statefun::{Address, Context, Effects, EgressIdentifier, FunctionRegistry, FunctionType};
//!
//! pub fn relay_to_kafka(_context: Context, message: StringValue) -> Effects {
//!     let mut effects = Effects::new();
//!
//!     effects.kafka_keyed_egress(
//!         EgressIdentifier::new("example", "greets"),
//!         "greeting",
//!         "the key",
//!         message,
//!     );
//!
//!     effects
//! }
//! ```

use protobuf::Message;

use statefun_proto::kafka_egress::KafkaProducerRecord;

use crate::{Effects, EgressIdentifier, GetTypename, Serializable, TypeSpec};

/// Extension trait for sending egress messages to Kafka using [Effects](crate::Effects).
pub trait KafkaEgress {
    /// Sends the given message to the Kafka topic `topic` via the egress specified using the
    /// `EgressIdentifier`.
    fn kafka_egress<T: Serializable<T>>(
        &mut self,
        type_spec: TypeSpec<T>,
        identifier: EgressIdentifier,
        topic: &str,
        value: &T,
    ) -> Result<(), String>;

    /// Sends the given message to the Kafka topic `topic` via the egress specified using the
    /// `EgressIdentifier`.
    ///
    /// This will set the given key on the message sent to record.
    fn kafka_keyed_egress<T: Serializable<T>>(
        &mut self,
        type_spec: TypeSpec<T>,
        identifier: EgressIdentifier,
        topic: &str,
        key: &str,
        value: &T,
    ) -> Result<(), String>;
}

impl KafkaEgress for Effects {
    fn kafka_egress<T: Serializable<T>>(
        &mut self,
        type_spec: TypeSpec<T>,
        identifier: EgressIdentifier,
        topic: &str,
        value: &T,
    ) -> Result<(), String> {
        let kafka_record = egress_record(topic, type_spec, value)?;

        // todo: what do we set this as? see Java SDK
        let type_spec: TypeSpec<KafkaProducerRecord> = TypeSpec::<KafkaProducerRecord>::new();

        self.egress(identifier, type_spec, &kafka_record)
    }

    fn kafka_keyed_egress<T: Serializable<T>>(
        &mut self,
        type_spec: TypeSpec<T>,
        identifier: EgressIdentifier,
        topic: &str,
        key: &str,
        value: &T,
    ) -> Result<(), String> {
        let mut kafka_record = egress_record(topic, type_spec, value)?;
        kafka_record.set_key(key.to_owned());

        let type_spec: TypeSpec<KafkaProducerRecord> = TypeSpec::<KafkaProducerRecord>::new();
        self.egress(identifier, type_spec, &kafka_record)
    }
}

impl GetTypename for KafkaProducerRecord {
    fn get_typename() -> &'static str {
        // todo: what do we set this as? see Java SDK
        "kafka/user-profile"
    }
}

impl Serializable<KafkaProducerRecord> for KafkaProducerRecord {
    fn serialize(&self, _typename: String) -> Result<Vec<u8>, String> {
        match self.write_to_bytes() {
            Ok(result) => Ok(result),
            Err(result) => Err(result.to_string()),
        }
    }

    fn deserialize(_typename: String, buffer: &Vec<u8>) -> Result<KafkaProducerRecord, String> {
        match KafkaProducerRecord::parse_from_bytes(buffer) {
            Ok(result) => Ok(result),
            Err(result) => Err(result.to_string()),
        }
    }
}

fn egress_record<T: Serializable<T>>(
    topic: &str,
    type_spec: TypeSpec<T>,
    value: &T,
) -> Result<KafkaProducerRecord, String> {
    let mut result = KafkaProducerRecord::new();
    result.set_topic(topic.to_owned());
    let serialized = value.serialize(type_spec.typename.to_string())?;
    result.set_value_bytes(serialized);
    Ok(result)
}
