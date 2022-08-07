use lapin::{options::QueueDeclareOptions, types::FieldTable, Channel, Queue};

pub const QUEUE_NAME: &str = "hello";
pub const CONSUMER_TAG: &str = "consumer";

pub async fn declare_queue(channel: &Channel) -> Queue {
    channel
        .queue_declare(
            QUEUE_NAME,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("declare queue error")
}
