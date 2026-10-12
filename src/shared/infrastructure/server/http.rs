use std::{io, net::SocketAddr};

use tokio::net::TcpListener;

use super::{AppState, router, shutdown};

pub struct Server {
    port: u16,
    state: AppState,
}

impl Server {
    pub fn new(port: u16, state: AppState) -> Self {
        Self { port, state }
    }

    pub async fn run(self) -> io::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("Listening on http://{addr}");

        axum::serve(listener, router::build(self.state))
            .with_graceful_shutdown(shutdown::signal())
            .await
    }
}
