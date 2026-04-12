use std::{future::Future, pin::Pin};

use poem::{EndpointExt, IntoEndpoint, middleware::Tracing, middleware::Cors};
use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};

pub struct Config<E>
where
    E: IntoEndpoint,
    E::Endpoint: 'static,
{
    pub service_url: String,
    pub port: u16,
    pub routes: E,
}

pub type HandlerFn = JoinHandle<Result<(), ServiceError>>;
pub type TeardownFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
pub type TeardownFn = Box<dyn FnOnce() -> TeardownFuture + Send>;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("poem error: {0}")]
    Poem(#[from] poem::Error),

    #[error("{task} failed: {source}")]
    TaskFailed {
        task: &'static str,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("{task} join failed: {source}")]
    TaskJoin {
        task: &'static str,
        #[source]
        source: JoinError,
    },
}

pub async fn service_setup<E>(
    config: Config<E>,
    mut handlers: Vec<(String, HandlerFn)>,
    teardown: Vec<TeardownFn>,
) -> Result<(), ServiceError>
where
    E: IntoEndpoint,
    E::Endpoint: Send + 'static,
{
    let server = server_setup(config);
    let signals = signals();

    tracing::info!(workers = handlers.len(), "service started");

    tokio::select! {
        res = signals => handle_join("Signals", res)?,
        res = server  => handle_join("Server",  res)?,
    }

    tracing::info!("shutting down workers");
    for (name, h) in handlers.iter_mut() {
        tracing::debug!(%name, "aborting");
        h.abort();
    }

    tracing::info!(teardown = teardown.len(), "running teardown");
    for f in teardown {
        f().await;
    }

    Ok(())
}

fn handle_join(
    task: &'static str,
    res: Result<Result<(), ServiceError>, JoinError>,
) -> Result<(), ServiceError> {
    match res {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(ServiceError::TaskFailed {
            task,
            source: Box::new(e),
        }),
        Err(e) => Err(ServiceError::TaskJoin { task, source: e }),
    }
}

pub fn make_teardown<F, Fut>(f: F) -> TeardownFn
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    Box::new(move || Box::pin(f()))
}

pub fn server_setup<E>(config: Config<E>) -> HandlerFn
where
    E: IntoEndpoint,
    E::Endpoint: Send + 'static,
{
    let Config {
        service_url,
        port,
        routes,
    } = config;
    let app = routes.into_endpoint().with(Cors::new()).with(Tracing);

    tokio::spawn(async move {
        tracing::info!("Server is running on {}", service_url);

        poem::Server::new(poem::listener::TcpListener::bind(format!("0.0.0.0:{port}")))
            .run(app)
            .await
            .map_err(ServiceError::from)
    })
}

pub fn signals() -> HandlerFn {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut sigterm = signal(SignalKind::terminate())?;
            let mut sigint = signal(SignalKind::interrupt())?;
            let mut sigquit = signal(SignalKind::quit())?;

            tokio::select! {
                _ = sigterm.recv() => tracing::info!("Received SIGTERM"),
                _ = sigint.recv()  => tracing::info!("Received SIGINT"),
                _ = sigquit.recv() => tracing::info!("Received SIGQUIT"),
            }
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await?;
            tracing::info!("Received Ctrl-C");
        }

        Ok(())
    })
}
