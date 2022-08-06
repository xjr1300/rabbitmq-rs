use common::{get_rabbitmq_address, set_default_logging_env};

fn main() {
    set_default_logging_env();

    tracing_subscriber::fmt::init();

    let address = get_rabbitmq_address();
}
