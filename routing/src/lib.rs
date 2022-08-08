use lapin::{options::ExchangeDeclareOptions, types::FieldTable, Channel, ExchangeKind};

pub const EXCHANGE_NAME: &str = "direct_log";

pub enum Severity {
    Info,
    Warn,
    Error,
}

impl TryFrom<&str> for Severity {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, <Severity as TryFrom<&str>>::Error> {
        match value {
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(format!("不明な重要度({})です。", value)),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "{}", "info"),
            Self::Warn => write!(f, "{}", "warn"),
            Self::Error => write!(f, "{}", "error"),
        }
    }
}

pub async fn declare_exchange(channel: &Channel) {
    let _ = channel
        .exchange_declare(
            EXCHANGE_NAME,
            ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await;
}
