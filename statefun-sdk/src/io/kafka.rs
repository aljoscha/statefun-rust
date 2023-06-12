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

use crate::{Effects, EgressIdentifier, ValueSpecBase};

/// Extension trait for sending egress messages to Kafka using [Effects](crate::Effects).
pub trait KafkaEgress {
    /// Sends the given message to the Kafka topic `topic` via the egress specified using the
    /// `EgressIdentifier`.
    fn kafka_egress<M: Message>(&mut self, identifier: EgressIdentifier, topic: &str, message: M);

    /// Sends the given message to the Kafka topic `topic` via the egress specified using the
    /// `EgressIdentifier`.
    ///
    /// This will set the given key on the message sent to record.
    fn kafka_keyed_egress<M: Message>(
        &mut self,
        identifier: EgressIdentifier,
        topic: &str,
        key: &str,
        message: M,
    );
}

impl KafkaEgress for Effects {
    fn kafka_egress<M: Message>(&mut self, identifier: EgressIdentifier, topic: &str, message: M) {
        let kafka_record = egress_record(topic, message);
        // todo: figure out the type name here, maybe just use a string??
        self.egress(
            identifier,
            ValueSpecBase::new("kafka", "kafka"),
            kafka_record,
        );
    }

    fn kafka_keyed_egress<M: Message>(
        &mut self,
        identifier: EgressIdentifier,
        topic: &str,
        key: &str,
        message: M,
    ) {
        let mut kafka_record = egress_record(topic, message);
        kafka_record.set_key(key.to_owned());
        self.egress(
            identifier,
            ValueSpecBase::new("kafka", "kafka"),
            kafka_record,
        );
    }
}

fn egress_record<M: Message>(topic: &str, value: M) -> KafkaProducerRecord {
    let mut result = KafkaProducerRecord::new();
    result.set_topic(topic.to_owned());
    result.set_value_bytes(value.write_to_bytes().expect("Could not serialize value."));
    result
}
