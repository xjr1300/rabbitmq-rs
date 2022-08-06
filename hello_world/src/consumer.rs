use futures_lite::stream::StreamExt;
use lapin::options::BasicConsumeOptions;
use lapin::{
    options::{BasicAckOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use tracing::info;

use tutorial_1::{CONNECT_URL, CONSUMER_TAG, QUEUE_NAME};

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let address = std::env::var("AMQP_ADDR").unwrap_or_else(|_| CONNECT_URL.into());

    async_global_executor::block_on(async {
        // RabbitMQに接続
        let conn = Connection::connect(&address, ConnectionProperties::default())
            .await
            .expect("[CONSUMER] connection error");
        info!("[CONSUMER] connected");

        // チャネルを作成
        let channel = conn.create_channel().await.expect("create channel error");
        info!(state=?conn.status().state());

        // デフォルトエクスチェンジに接続したキューを定義
        let queue = channel
            .queue_declare(
                QUEUE_NAME,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("[CONSUMER] declare queue error");
        info!(state=?conn.status().state());
        info!(?queue, "[CONSUMER] declared queue");

        // キューにメッセージが到着することを待ち、メッセージを処理するコンシューマーを作成
        info!("[CONSUMER] will consume");
        let mut consumer = channel
            .basic_consume(
                QUEUE_NAME,
                CONSUMER_TAG,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("[CONSUMER] basic consume error");
        info!(state=?conn.status().state());

        // コンシューマーが、キューにメッセージが到着することを待ち、メッセージを処理
        while let Some(delivery) = consumer.next().await {
            info!(message=?delivery, "[CONSUMER] received message");
            let message = String::from_utf8(delivery.as_ref().unwrap().data.clone()).unwrap();
            info!("{}", format!("[CONSUMER] `{}` received", message));
            // メッセージを正常に処理したら、RabbitMQにメッセージを処理したことを示す肯定応答を返却
            if let Ok(delivery) = delivery {
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("[CONSUMER] basic ack error");
                info!("[CONSUMER] return ack");
            }
        }
    });
}
