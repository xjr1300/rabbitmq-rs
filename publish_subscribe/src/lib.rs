use lapin::{options::ExchangeDeclareOptions, types::FieldTable, Channel, ExchangeKind};

pub const EXCHANGE_NAME: &str = "logs";

pub async fn declare_exchange(channel: &Channel) {
    channel
        .exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("exchange error")
}
