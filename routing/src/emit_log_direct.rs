use lapin::{options::BasicPublishOptions, BasicProperties};
use tracing::info;

use common::{connect, set_default_logging_env};

use routing::{declare_exchange, Severity, EXCHANGE_NAME};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Severity didn't find in arguments.")
    }

    // 重要度をコマンドラインから取得
    let severity: Severity = Severity::try_from(args[1].as_ref()).unwrap();

    // 送信するメッセージをコマンドラインから取得
    let message = if args.len() == 2 {
        "Hello World!".to_string()
    } else {
        args[2..].join(" ")
    };

    async_global_executor::block_on(async {
        let conn = connect().await;
        info!("connected");

        let channel = conn.create_channel().await.expect("create channel error");

        // ダイレクトエクスチェンジを作成
        declare_exchange(&channel).await;

        info!("will publish");
        channel
            .basic_publish(
                EXCHANGE_NAME,
                &format!("{}", severity),
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .expect("basic publish error");
        info!("send `{}` at `{}`", message, severity);
    })
}
