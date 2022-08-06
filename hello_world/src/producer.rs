use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use tracing::info;

use common::{set_default_logging_env, RABBITMQ_URL};

use hello_world::QUEUE_NAME;

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    let address = std::env::var("AMQP_ADDR").unwrap_or_else(|_| RABBITMQ_URL.into());

    async_global_executor::block_on(async {
        // RabbitMQに接続
        let conn = Connection::connect(&address, ConnectionProperties::default())
            .await
            .expect("connection error");
        info!("connected");

        // チャネルを作成
        let channel = conn.create_channel().await.expect("create channel error");
        info!(state=?conn.status().state());

        // キューを定義
        let queue = channel
            .queue_declare(
                QUEUE_NAME,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("declare queue error");
        info!(state=?conn.status().state());
        info!(?queue, "declared queue");

        info!("publish");
        let payload = b"Hello world!";
        let _confirm = channel
            .basic_publish(
                "",         // デフォルトエクスチェンジ
                QUEUE_NAME, // ルーティングキー: デフォルトエクスチェンジの場合、ルーティングキーと同じ名前のキューに発行
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await
            .expect("basic publish error");
    });
}
