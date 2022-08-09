use lapin::{options::ExchangeDeclareOptions, types::FieldTable, Channel, ExchangeKind};

pub const EXCHANGE_NAME: &str = "topic_logs";

pub async fn declare_exchange(channel: &Channel) {
    let _ = channel
        .exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;
}
