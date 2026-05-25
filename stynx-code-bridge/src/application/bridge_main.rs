use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use stynx_code_errors::AppResult;
use crate::domain::session::BridgeSession;
use crate::infrastructure::websocket_transport::WebSocketTransport;
use crate::application::bridge_messaging::{BridgeMessageHandler, DefaultBridgeHandler};

pub struct BridgeServer {
    pub port: u16,
    pub sessions: Arc<RwLock<HashMap<String, BridgeSession>>>,
}

impl BridgeServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(port: u16) -> AppResult<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| stynx_code_errors::AppError::Internal(anyhow::anyhow!(e)))?;
        tracing::info!(addr, "bridge server listening");

        loop {
            let (stream, peer) = listener.accept().await
                .map_err(|e| stynx_code_errors::AppError::Internal(anyhow::anyhow!(e)))?;
            tracing::info!(%peer, "new connection");

            tokio::spawn(async move {
                let transport = WebSocketTransport::new();
                match transport.accept_connection(stream).await {
                    Ok(mut conn) => {
                        let handler = DefaultBridgeHandler;
                        loop {
                            match conn.recv().await {
                                Some(msg) => {
                                    match handler.handle_message(msg).await {
                                        Ok(response) => {
                                            if let Err(e) = conn.send(response).await {
                                                tracing::error!(?e, "send error");
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!(?e, "handler error");
                                            break;
                                        }
                                    }
                                }
                                None => {
                                    tracing::info!("connection closed");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(?e, "websocket upgrade failed");
                    }
                }
            });
        }
    }

    pub async fn stop(&self) {
        tracing::info!("bridge server stopping");
        self.sessions.write().await.clear();
    }
}
