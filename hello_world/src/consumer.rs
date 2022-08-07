use futures_lite::stream::StreamExt;
use lapin::options::BasicConsumeOptions;
use lapin::{options::BasicAckOptions, types::FieldTable};
use tracing::info;

use common::{connect, set_default_logging_env};

use hello_world::{declare_queue, CONSUMER_TAG, QUEUE_NAME};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    async_global_executor::block_on(async {
        // RabbitMQに接続
        let conn = connect().await;
        info!("connected");

        // チャネルを作成
        let channel = conn.create_channel().await.expect("create channel error");
        info!(state=?conn.status().state());

        // キューを定義
        let queue = declare_queue(&channel).await;
        info!(state=?conn.status().state());
        info!(?queue, "declared queue");

        // キューにメッセージが到着することを待ち、メッセージを処理するコンシューマーを作成
        info!("will consume");
        let mut consumer = channel
            .basic_consume(
                QUEUE_NAME,
                CONSUMER_TAG,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("basic consume error");
        info!(state=?conn.status().state());

        // コンシューマーが、キューにメッセージが到着することを待ち、メッセージを処理
        while let Some(delivery) = consumer.next().await {
            info!(message=?delivery, "received message");
            let message = String::from_utf8(delivery.as_ref().unwrap().data.clone()).unwrap();
            info!("{}", format!("`{}` received", message));
            // メッセージを正常に処理したら、RabbitMQにメッセージを処理したことを示す肯定応答を返却
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
