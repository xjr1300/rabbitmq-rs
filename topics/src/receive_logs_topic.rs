use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
};
use topics::{declare_exchange, EXCHANGE_NAME};
use tracing::info;

use common::{connect, set_default_logging_env};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    // バインディングキーをコマンドライン引数から取得
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Binding Key didn't find in arguments.");
    }
    let binding_keys: Vec<String> = args.drain(1..).collect();

    async_global_executor::block_on(async {
        let conn = connect().await;
        info!("connected");

        let channel = conn.create_channel().await.expect("create queue error");

        // トピックエクスチェンジを作成
        declare_exchange(&channel).await;

        // 名前を指定せずに、永続的なキューを定義
        let queue_options = QueueDeclareOptions {
            exclusive: true,
            ..Default::default()
        };
        let queue = channel
            .queue_declare("", queue_options, FieldTable::default())
            .await
            .expect("declare queue error");

        // バインディングキーでエクスチェンジとキューをバインド
        for binding_key in binding_keys.iter() {
            channel
                .queue_bind(
                    queue.name().as_str(),
                    EXCHANGE_NAME,
                    binding_key,
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .expect("queue bind error");
        }

        info!("will consume");
        let mut consumer = channel
            .basic_consume(
                queue.name().as_str(),
                "receive_logs_topic",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("basic consume error");

        println!("[*] Waiting for logs. To exit press Ctrl + C");
        while let Some(delivery) = consumer.next().await {
            info!(message=?delivery, "received message");
            if let Ok(delivery) = delivery {
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("basic ack error");
                let routing_key = delivery.routing_key.clone();
                let message = String::from_utf8(delivery.data.clone()).unwrap();
                println!("[x] {}:{}", routing_key, message);
            }
        }
    });
}
