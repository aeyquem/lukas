use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use std::{future::Future, time::Duration};
use thiserror::Error;
use tokio::time::timeout;

use crate::config::Config;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("PostgreSQL connection failed")]
    Connection,
    #[error("database migrations failed")]
    Migration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupDeadlineExceeded;

pub async fn with_startup_deadline<F, T, E>(
    pool: &PgPool,
    deadline: Duration,
    operation: F,
) -> Result<Result<T, E>, StartupDeadlineExceeded>
where
    F: Future<Output = Result<T, E>>,
{
    match timeout(deadline, operation).await {
        Ok(result) => Ok(result),
        Err(_) => {
            pool.close().await;
            Err(StartupDeadlineExceeded)
        }
    }
}

pub async fn connect_pool(config: &Config) -> Result<PgPool, ConnectError> {
    let pool_connection = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect(&config.database_url);
    let pool = timeout(config.connect_timeout, pool_connection)
        .await
        .map_err(|_| ConnectError::Connection)?
        .map_err(|_| ConnectError::Connection)?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), ConnectError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|_| ConnectError::Migration)
}

#[cfg(test)]
mod tests {
    use std::{future::pending, time::Duration};

    use sqlx::postgres::PgPoolOptions;

    use super::{StartupDeadlineExceeded, with_startup_deadline};

    #[tokio::test]
    async fn startup_deadline_cancels_the_operation_and_closes_the_pool() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://lukas:lukas@127.0.0.1:1/lukas")
            .unwrap();

        let result = with_startup_deadline(&pool, Duration::from_millis(10), async {
            pending::<Result<(), ()>>().await
        })
        .await;

        assert_eq!(result, Err(StartupDeadlineExceeded));
        assert!(pool.is_closed());
    }
}
