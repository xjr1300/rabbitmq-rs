use lapin::{options::BasicPublishOptions, BasicProperties};
use topics::{declare_exchange, EXCHANGE_NAME};
use tracing::info;

use common::{connect, set_default_logging_env};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    // 送信するメッセージのルーティングキーをコマンドラインから取得
    let args: Vec<String> = std::env::args().collect();
    let routing_key = if args.len() < 2 {
        "anonymous.info".to_string()
    } else {
        args[1].clone()
    };
    // 送信するメッセージをコマンドラインから取得
    let message = if args.len() < 3 {
        "Hello World!".to_string()
    } else {
        args[2..].join(" ")
    };

    async_global_executor::block_on(async {
        let conn = connect().await;
        info!("connected");

        let channel = conn.create_channel().await.expect("create channel error");

        // トピックエクスチェンジを作成
        declare_exchange(&channel).await;

        info!("will publish");
        channel
            .basic_publish(
                EXCHANGE_NAME,
                &routing_key,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .expect("basic publish error");
        info!("send `{}` with `{}` routing key", message, routing_key);
    });
}
