use futures_lite::stream::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
};
use tracing::info;

use common::{connect, set_default_logging_env};

use publish_subscribe::{declare_exchange, EXCHANGE_NAME};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    async_global_executor::block_on(async {
        let conn = connect().await;
        info!("connected");

        let channel = conn.create_channel().await.expect("create channel error");
        info!(state=?conn.status().state());

        // logsファンアウトエクスチェンジを作成
        declare_exchange(&channel).await;

        // 名前を指定せずに、排他的なキューを作成
        let queue_options = QueueDeclareOptions {
            exclusive: true,
            ..Default::default()
        };
        let queue = channel
            .queue_declare("", queue_options, FieldTable::default())
            .await
            .expect("declare queue error");
        info!("queue name is `{}`", queue.name());
        println!("[*] Waiting for logs. To exist press Ctrl + C");

        // チャンネルとキューをバインド
        let _ = channel
            .queue_bind(
                queue.name().as_str(),
                EXCHANGE_NAME,
                "",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await;

        info!("will consume");
        let mut consumer = channel
            .basic_consume(
                queue.name().as_str(),
                "receive_log",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("basic consume error");
        info!(state=?conn.status().state());

        while let Some(delivery) = consumer.next().await {
            info!(message=?delivery, "received message");
            let message = String::from_utf8(delivery.as_ref().unwrap().data.clone()).unwrap();
            info!("`{}` received", message);
            if let Ok(delivery) = delivery {
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("basic ack error");
                info!("return ack");
            }
        }
    });
}
