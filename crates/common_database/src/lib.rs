use anyhow::{ensure, Context};
use deadpool_postgres::{Config, Pool};

/// Create Database Pool
///
/// This function internally reads the following environment variables:
/// - `POSTGRES_PASSWORD` (required)
/// - `POSTGRES_USER` (default is `postgres`)
/// - `POSTGRES_DB` (default is the same as `POSTGRES_USER`)
/// - `POSTGRES_HOST` (default is `postgres`)
/// - `POSTGRES_PORT` (default is `5432`)
///
/// You sholud create pool once and use it as a singleton in your application.
pub fn create_db_pool() -> anyhow::Result<Pool> {
    ensure!(
        envmnt::exists("POSTGRES_PASSWORD"),
        "Environment variable POSTGRES_PASSWORD not provided"
    );

    let postgres_password = envmnt::get_or_panic("POSTGRES_PASSWORD"); // will never panic
    let postgres_user = envmnt::get_or("POSTGRES_USER", "postgres");
    let postgres_db = envmnt::get_or("POSTGRES_DB", &postgres_user);
    let postgres_host = envmnt::get_or("POSTGRES_HOST", "postgres");
    let postgres_port = envmnt::get_u16("POSTGRES_PORT", 5432);

    let mut config = Config::new();
    config.dbname = Some(postgres_db);
    config.host = Some(postgres_host);
    config.port = Some(postgres_port);
    config.user = Some(postgres_user);
    config.password = Some(postgres_password);

    config
        .create_pool(None, tokio_postgres::NoTls)
        .with_context(|| "Error during Postgres Pool creation")
}
