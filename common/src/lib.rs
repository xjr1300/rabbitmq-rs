pub const RABBITMQ_URL: &str = "amqp://127.0.0.1:5672/%2f";

const RUST_LOG_KEY: &str = "RUST_LOG";
const RUST_LOG_DEFAULT_VALUE: &str = "info";

pub fn set_default_logging_env() {
    if std::env::var(RUST_LOG_KEY).is_err() {
        std::env::set_var(RUST_LOG_KEY, RUST_LOG_DEFAULT_VALUE);
    }
}

pub fn get_rabbitmq_address() -> String {
    std::env::var("AMQP_ADDR").unwrap_or_else(|_| RABBITMQ_URL.into())
}
