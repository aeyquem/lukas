use std::{future::Future, future::IntoFuture, io, time::Duration};

use axum::{Router, serve};
use tokio::{net::TcpListener, sync::oneshot, time::timeout};

pub async fn serve_until_shutdown<F>(
    listener: TcpListener,
    app: Router,
    shutdown_signal: F,
    shutdown_timeout: Duration,
) -> io::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        })
        .into_future();
    tokio::pin!(server);

    tokio::select! {
        result = &mut server => result,
        _ = shutdown_signal => {
            let _ = shutdown_tx.send(());
            match timeout(shutdown_timeout, &mut server).await {
                Ok(result) => result,
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "graceful shutdown exceeded the configured timeout",
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        net::SocketAddr,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use axum::{Router, routing::get};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
        sync::{Semaphore, oneshot},
        task::JoinHandle,
        time::{sleep, timeout},
    };

    use super::serve_until_shutdown;

    async fn server_address() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        (listener, address)
    }

    async fn send_request(address: SocketAddr) -> io::Result<Vec<u8>> {
        let mut stream = TcpStream::connect(address).await?;
        stream
            .write_all(b"GET /hold HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await?;
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;
        Ok(response)
    }

    async fn wait_for_requests(count: &AtomicUsize) {
        timeout(Duration::from_secs(1), async {
            while count.load(Ordering::SeqCst) == 0 {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
    }

    fn held_router(requests: Arc<AtomicUsize>, release: Arc<Semaphore>) -> Router {
        Router::new().route(
            "/hold",
            get(move || {
                let requests = requests.clone();
                let release = release.clone();
                async move {
                    requests.fetch_add(1, Ordering::SeqCst);
                    let permit = release.acquire().await.unwrap();
                    permit.forget();
                    "done"
                }
            }),
        )
    }

    async fn wait_server(handle: JoinHandle<io::Result<()>>) -> io::Result<()> {
        timeout(Duration::from_secs(2), handle)
            .await
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn shutdown_drains_active_request_and_stops_accepting_new_work() {
        let (listener, address) = server_address().await;
        let requests = Arc::new(AtomicUsize::new(0));
        let release = Arc::new(Semaphore::new(0));
        let app = held_router(requests.clone(), release.clone());
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = tokio::spawn(serve_until_shutdown(
            listener,
            app,
            async move {
                let _ = shutdown_rx.await;
            },
            Duration::from_secs(1),
        ));

        let first_request = tokio::spawn(send_request(address));
        wait_for_requests(&requests).await;
        shutdown_tx.send(()).unwrap();
        sleep(Duration::from_millis(50)).await;

        let second_request = timeout(Duration::from_millis(100), send_request(address)).await;
        match second_request {
            Err(_) | Ok(Err(_)) => {}
            Ok(Ok(_)) => panic!("new request completed during shutdown"),
        }
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        release.add_permits(1);
        let first_response = timeout(Duration::from_secs(1), first_request)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(String::from_utf8_lossy(&first_response).contains("200 OK"));

        wait_server(server).await.unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn shutdown_exits_by_deadline_if_request_never_drains() {
        let (listener, address) = server_address().await;
        let requests = Arc::new(AtomicUsize::new(0));
        let release = Arc::new(Semaphore::new(0));
        let app = held_router(requests.clone(), release);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = tokio::spawn(serve_until_shutdown(
            listener,
            app,
            async move {
                let _ = shutdown_rx.await;
            },
            Duration::from_millis(150),
        ));

        let client = tokio::spawn(send_request(address));
        wait_for_requests(&requests).await;
        shutdown_tx.send(()).unwrap();

        let result = wait_server(server).await;
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
        client.abort();
    }

    #[tokio::test]
    async fn server_remains_available_past_drain_timeout_until_signaled() {
        let (listener, address) = server_address().await;
        let app = Router::new().route("/hold", get(|| async { "done" }));
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = tokio::spawn(serve_until_shutdown(
            listener,
            app,
            async move {
                let _ = shutdown_rx.await;
            },
            Duration::from_millis(50),
        ));

        sleep(Duration::from_millis(125)).await;
        let response = send_request(address).await.unwrap();
        assert!(String::from_utf8_lossy(&response).contains("200 OK"));

        shutdown_tx.send(()).unwrap();
        wait_server(server).await.unwrap();
    }
}
