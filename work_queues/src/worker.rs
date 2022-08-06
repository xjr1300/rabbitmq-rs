use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
};
use tracing::info;

use common::set_default_logging_env;

use work_queues::{connect, declare_queue, QUEUE_NAME};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    async_global_executor::block_on(async {
        let conn = connect().await;

        let channel = conn.create_channel().await.expect("create channel error");
        info!(state=?conn.status().state());

        let queue = declare_queue(&channel).await;
        info!(state=?conn.status().state());
        info!(?queue, "declare queue");

        info!("will consume");
        let mut consumer = channel
            .basic_consume(
                QUEUE_NAME,
                "consume_worker_tag",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("basic consume error");
        info!(state=?conn.status().state());

        while let Some(delivery) = consumer.next().await {
            info!(message=?delivery, "receive message");
            let message = String::from_utf8(delivery.as_ref().unwrap().data.clone()).unwrap();
            info!("{}", format!("`{}` received", message));
            if let Ok(delivery) = delivery {
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .expect("basic ack error");
                info!("return ack");
            }
        }
    })
}
