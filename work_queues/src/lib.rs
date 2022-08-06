use common::get_rabbitmq_address;
use lapin::{
    options::QueueDeclareOptions, types::FieldTable, Channel, Connection, ConnectionProperties,
    Queue,
};

pub const QUEUE_NAME: &str = "task_queue";

pub async fn connect() -> Connection {
    let address = get_rabbitmq_address();
    Connection::connect(&address, ConnectionProperties::default())
        .await
        .expect("connection error")
}

pub async fn declare_queue(channel: &Channel) -> Queue {
    let mut queue_options = QueueDeclareOptions::default();
    queue_options.durable = true;
    channel
        .queue_declare(QUEUE_NAME, queue_options, FieldTable::default())
        .await
        .expect("declare queue error")
}
