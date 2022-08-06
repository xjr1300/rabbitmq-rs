use lapin::{options::QueueDeclareOptions, types::FieldTable, Channel, Queue};

pub const QUEUE_NAME: &str = "task_queue";

pub async fn declare_queue(channel: &Channel) -> Queue {
    let mut queue_options = QueueDeclareOptions::default();
    queue_options.durable = true;
    channel
        .queue_declare(QUEUE_NAME, queue_options, FieldTable::default())
        .await
        .expect("declare queue error")
}
