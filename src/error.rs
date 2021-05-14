use thiserror::Error;

#[derive(Error, Debug)]
pub enum AriadneErrors {
    #[error("A database error occurred.")]
    ClickHouseError(#[from] clickhouse_rs::errors::Error),
}
