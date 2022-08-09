use futures_lite::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
};

use tracing::info;

use common::{connect, set_default_logging_env};

use routing::{declare_exchange, Severity, EXCHANGE_NAME};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    // 受信するログの重要度を取得
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Severity didn't find in arguments.")
    }

    // コマンドライン引数から受信するログの重要度を複数取得
    let severities = args[1..]
        .iter()
        .map(|value| Severity::try_from(value.as_str()).unwrap());

    async_global_executor::block_on(async {
        let conn = connect().await;
        info!("connected");

        let channel = conn.create_channel().await.expect("create channel error");

        // ダイレクトエクスチェンジを作成
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

        // キューとエクスチェンジを重要度を示すルーティングキーでバインド
        for severity in severities {
            channel
                .queue_bind(
                    queue.name().as_str(),
                    EXCHANGE_NAME,
                    &format!("{}", severity),
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .expect("binding error");
        }

        info!("will consume");
        let mut consumer = channel
            .basic_consume(
                queue.name().as_str(),
                "receive_logs_direct",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("basic consume error");

        println!("[*] Waiting for logs. To exit press Ctrl + C");
        while let Some(delivery) = consumer.next().await {
            info!(message=?delivery, "received message");
            if let Ok(delivery) = delivery {
                let routing_key = delivery.routing_key.clone();
                let message = String::from_utf8(delivery.data.clone()).unwrap();
                println!("[x] {}:{}", routing_key, message);
            }
        }
    });
}
