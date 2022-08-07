use lapin::{options::QueueDeclareOptions, types::FieldTable, Channel, Queue};

pub const QUEUE_NAME: &str = "task_queue";

pub async fn declare_queue(channel: &Channel) -> Queue {
    let queue_options = QueueDeclareOptions {
        durable: true,
        ..Default::default()
    };
    channel
        .queue_declare(QUEUE_NAME, queue_options, FieldTable::default())
        .await
        .expect("declare queue error")
}
