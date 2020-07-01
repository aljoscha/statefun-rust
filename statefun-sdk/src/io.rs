pub mod kafka {
    use protobuf::Message;

    use statefun_protos::kafka_egress::KafkaProducerRecord;

    pub fn egress_record<M: Message>(topic: &str, value: M) -> KafkaProducerRecord {
        let mut result = KafkaProducerRecord::new();
        result.set_topic(topic.to_owned());
        result.set_value_bytes(value.write_to_bytes().expect("Could not serialize value."));
        result
    }

    pub fn keyed_egress_record<M: Message>(
        topic: &str,
        key: &str,
        value: M,
    ) -> KafkaProducerRecord {
        let mut result = egress_record(topic, value);
        result.set_key(key.to_owned());
        result
    }
}
