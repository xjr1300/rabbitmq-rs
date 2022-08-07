use lapin::{options::BasicPublishOptions, BasicProperties};
use tracing::info;

use common::{connect, set_default_logging_env};

use publish_subscribe::{declare_exchange, EXCHANGE_NAME};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    async_global_executor::block_on(async {
        let conn = connect().await;
        info!("connected");

        let channel = conn.create_channel().await.expect("create channel error ");

        //
        // 他のチュートリアルと異なり、ランダムな名前を持つキューを利用するため、
        // プロデューサーはキューを定義していない
        //

        // logsファンアウトエクスチェンジを作成
        declare_exchange(&channel).await;

        // 送信するメッセージを作成
        let args: Vec<String> = std::env::args().collect();
        let message = if args.is_empty() {
            "info: Hello World!".to_string()
        } else {
            args.join(" ")
        };

        info!("will publish");
        channel
            .basic_publish(
                EXCHANGE_NAME,
                "",
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .expect("basic publish error");
        info!("send `{}`", message);
    });
}
