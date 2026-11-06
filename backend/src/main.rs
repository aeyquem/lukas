use std::{env, net::TcpListener as StdTcpListener, process};

use lukas_api::{
    config::{Config, ConfigError},
    database,
    health::router,
    server,
    workspace::bootstrap,
};
use sqlx::PgPool;
use thiserror::Error;
use tokio::{net::TcpListener, signal};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Error)]
enum StartupError {
    #[error("invalid runtime configuration: {0}")]
    Configuration(#[from] ConfigError),
    #[error("could not connect to PostgreSQL")]
    DatabaseConnection,
    #[error("database migration failed")]
    Migration,
    #[error("database migrations and workspace bootstrap exceeded their startup deadline")]
    DatabaseStartupTimeout,
    #[error("owner workspace bootstrap failed")]
    WorkspaceBootstrap,
    #[error("could not bind the API listener")]
    Listener,
    #[error("HTTP server failed or exceeded the shutdown deadline")]
    Server,
    #[error("unsupported command line argument")]
    Arguments,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupCommand {
    Serve,
    MigrateOnly,
    RotateDatabaseGeneration,
}

enum StartupPlan {
    MigrateOnly,
    RotateDatabaseGeneration(Uuid),
    Serve(Uuid),
}

fn startup_plan(
    command: StartupCommand,
    workspace_id: Option<Uuid>,
) -> Result<StartupPlan, StartupError> {
    match command {
        StartupCommand::MigrateOnly => Ok(StartupPlan::MigrateOnly),
        StartupCommand::RotateDatabaseGeneration => workspace_id
            .map(StartupPlan::RotateDatabaseGeneration)
            .ok_or(StartupError::WorkspaceBootstrap),
        StartupCommand::Serve => workspace_id
            .map(StartupPlan::Serve)
            .ok_or(StartupError::WorkspaceBootstrap),
    }
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn startup_command() -> Result<StartupCommand, StartupError> {
    match env::args().skip(1).next().as_deref() {
        None => Ok(StartupCommand::Serve),
        Some("--migrate-only") if env::args().len() == 2 => Ok(StartupCommand::MigrateOnly),
        Some("--rotate-database-generation") if env::args().len() == 2 => {
            Ok(StartupCommand::RotateDatabaseGeneration)
        }
        _ => Err(StartupError::Arguments),
    }
}

async fn connect(config: &Config) -> Result<PgPool, StartupError> {
    database::connect_pool(config)
        .await
        .map_err(|error| match error {
            database::ConnectError::Connection => StartupError::DatabaseConnection,
            database::ConnectError::Migration => StartupError::Migration,
        })
}

async fn run() -> Result<(), StartupError> {
    let command = startup_command()?;
    let config = Config::from_env()?;
    let pool = connect(&config).await?;

    let preparation =
        database::with_startup_deadline(&pool, config.database_startup_timeout, async {
            database::run_migrations(&pool)
                .await
                .map_err(|_| StartupError::Migration)?;

            if command == StartupCommand::MigrateOnly {
                Ok(None)
            } else {
                bootstrap(&pool)
                    .await
                    .map(Some)
                    .map_err(|_| StartupError::WorkspaceBootstrap)
            }
        })
        .await;

    let workspace_id = match preparation {
        Ok(result) => result?,
        Err(_) => return Err(StartupError::DatabaseStartupTimeout),
    };

    match startup_plan(command, workspace_id)? {
        StartupPlan::Serve(workspace_id) => {
            let app = router(pool.clone(), workspace_id, config.clone());

            let standard_listener =
                StdTcpListener::bind(config.bind_address).map_err(|_| StartupError::Listener)?;
            standard_listener
                .set_nonblocking(true)
                .map_err(|_| StartupError::Listener)?;
            let listener =
                TcpListener::from_std(standard_listener).map_err(|_| StartupError::Listener)?;

            info!(address = %config.bind_address, workspace_id = %workspace_id, "API is ready");
            server::serve_until_shutdown(listener, app, shutdown_signal(), config.shutdown_timeout)
                .await
                .map_err(|_| StartupError::Server)?;
        }
        StartupPlan::MigrateOnly => info!("database migrations completed"),
        StartupPlan::RotateDatabaseGeneration(workspace_id) => {
            database::rotate_installation_generation(&pool, workspace_id)
                .await
                .map_err(|_| StartupError::WorkspaceBootstrap)?;
            info!(workspace_id = %workspace_id, "installation generation rotated after restore");
        }
    }

    pool.close().await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = signal::ctrl_c().await {
            error!(%error, "failed to install Ctrl-C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => error!(%error, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{StartupCommand, StartupPlan, startup_plan};

    #[test]
    fn migrate_only_plan_never_enters_listener_startup_branch() {
        assert!(matches!(
            startup_plan(StartupCommand::MigrateOnly, None),
            Ok(StartupPlan::MigrateOnly)
        ));
        assert!(matches!(
            startup_plan(StartupCommand::Serve, Some(Uuid::nil())),
            Ok(StartupPlan::Serve(_))
        ));
        assert!(startup_plan(StartupCommand::Serve, None).is_err());
        assert!(matches!(
            startup_plan(StartupCommand::RotateDatabaseGeneration, Some(Uuid::nil())),
            Ok(StartupPlan::RotateDatabaseGeneration(_))
        ));
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(error) = run().await {
        // Error values are intentionally generic so a database URL or other
        // secret can never be echoed into logs or terminal output.
        error!(reason = %error, "startup or server operation failed");
        process::exit(1);
    }
}
