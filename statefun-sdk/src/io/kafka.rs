//! Provides [KafkaEgress](crate::io::kafka::KafkaEgress) for sending egress messages to Kafka.

use protobuf::Message;

use statefun_proto::kafka_egress::KafkaProducerRecord;

use crate::{Effects, EgressIdentifier, GetTypename, Serializable};

/// Extension trait for sending egress messages to Kafka using [Effects](crate::Effects).
pub trait KafkaEgress {
    /// Sends the given message to the Kafka topic `topic` via the egress specified using the
    /// `EgressIdentifier`.
    fn kafka_egress<T: Serializable<T> + GetTypename>(
        &mut self,
        identifier: EgressIdentifier,
        topic: &str,
        value: &T,
    ) -> Result<(), String>;

    /// Sends the given message to the Kafka topic `topic` via the egress specified using the
    /// `EgressIdentifier`.
    ///
    /// This will set the given key on the message sent to record.
    fn kafka_keyed_egress<T: Serializable<T> + GetTypename>(
        &mut self,
        identifier: EgressIdentifier,
        topic: &str,
        key: &str,
        value: &T,
    ) -> Result<(), String>;
}

impl KafkaEgress for Effects {
    fn kafka_egress<T: Serializable<T> + GetTypename>(
        &mut self,
        identifier: EgressIdentifier,
        topic: &str,
        value: &T,
    ) -> Result<(), String> {
        let kafka_record = egress_record(topic, value)?;
        self.egress(identifier, &kafka_record)
    }

    fn kafka_keyed_egress<T: Serializable<T> + GetTypename>(
        &mut self,
        identifier: EgressIdentifier,
        topic: &str,
        key: &str,
        value: &T,
    ) -> Result<(), String> {
        let mut kafka_record = egress_record(topic, value)?;
        kafka_record.set_key(key.to_owned());
        self.egress(identifier, &kafka_record)
    }
}

impl GetTypename for KafkaProducerRecord {
    fn get_typename() -> &'static str {
        "type.googleapis.com/io.statefun.sdk.egress.KafkaProducerRecord"
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

fn egress_record<T: Serializable<T> + GetTypename>(
    topic: &str,
    value: &T,
) -> Result<KafkaProducerRecord, String> {
    let mut result = KafkaProducerRecord::new();
    result.set_topic(topic.to_owned());
    let serialized = value.serialize(T::get_typename().to_string())?;
    result.set_value_bytes(serialized);
    Ok(result)
}
