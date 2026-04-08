//! WebSocket broadcaster for real-time task updates
//!
//! Provides a broadcast channel for distributing events to connected WebSocket clients.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Maximum number of events to buffer in the broadcast channel
const BROADCAST_CAPACITY: usize = 256;

/// WebSocket events sent to connected clients
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    /// A new task was created
    TaskCreated {
        id: String,
        title: String,
    },
    /// A task was updated (status change or content update)
    TaskUpdated {
        id: String,
        status: String,
    },
    /// A task was deleted
    TaskDeleted {
        id: String,
    },
    /// A new question was created on a task
    QuestionCreated {
        id: String,
        task_id: String,
    },
    /// A question was answered
    QuestionAnswered {
        id: String,
    },
    /// A new comment was added to a task
    CommentCreated {
        id: String,
        task_id: String,
    },
}

/// Broadcaster manages WebSocket connections and broadcasts events
#[derive(Debug, Clone)]
pub struct Broadcaster {
    /// Channel for broadcasting events to all connected clients
    tx: broadcast::Sender<WsEvent>,
    /// Set of connected client IDs for tracking
    clients: Arc<RwLock<HashSet<Uuid>>>,
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Broadcaster {
    /// Create a new broadcaster instance
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            tx,
            clients: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Broadcast an event to all connected WebSocket clients
    pub fn broadcast(&self, event: WsEvent) {
        match &event {
            WsEvent::TaskCreated { id, title } => {
                info!("Broadcasting TaskCreated: {} - {}", id, title);
            }
            WsEvent::TaskUpdated { id, status } => {
                info!("Broadcasting TaskUpdated: {} -> {}", id, status);
            }
            WsEvent::TaskDeleted { id } => {
                info!("Broadcasting TaskDeleted: {}", id);
            }
            WsEvent::QuestionCreated { id, task_id } => {
                info!("Broadcasting QuestionCreated: {} on task {}", id, task_id);
            }
            WsEvent::QuestionAnswered { id } => {
                info!("Broadcasting QuestionAnswered: {}", id);
            }
            WsEvent::CommentCreated { id, task_id } => {
                info!("Broadcasting CommentCreated: {} on task {}", id, task_id);
            }
        }

        if let Err(e) = self.tx.send(event) {
            debug!("No connected clients to receive event: {}", e);
        }
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Handle WebSocket upgrade request
    pub fn handle_ws(&self, ws: WebSocketUpgrade) -> Response {
        let broadcaster = self.clone();
        ws.on_upgrade(move |socket| Self::handle_connection(broadcaster, socket))
    }

    /// Handle a new WebSocket connection
    async fn handle_connection(broadcaster: Broadcaster, socket: WebSocket) {
        let client_id = Uuid::new_v4();

        {
            let mut clients = broadcaster.clients.write().await;
            clients.insert(client_id);
            info!(
                "WebSocket client connected: {} (total: {})",
                client_id,
                clients.len()
            );
        }

        let (sender, mut receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));
        let mut rx = broadcaster.tx.subscribe();
        let sender_for_recv = sender.clone();

        let recv_task = async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Ping(data)) => {
                        debug!("Received ping from client {}", client_id);
                        let mut sender = sender_for_recv.lock().await;
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("Received pong from client {}", client_id);
                    }
                    Ok(Message::Close(_)) => {
                        debug!("Client {} requested close", client_id);
                        break;
                    }
                    Ok(Message::Text(text)) => {
                        debug!("Received text from client {}: {}", client_id, text);
                    }
                    Ok(Message::Binary(data)) => {
                        debug!(
                            "Received binary from client {}: {} bytes",
                            client_id,
                            data.len()
                        );
                    }
                    Err(e) => {
                        warn!("Error receiving from client {}: {}", client_id, e);
                        break;
                    }
                }
            }
        };

        let send_task = async move {
            while let Ok(event) = rx.recv().await {
                let json = match serde_json::to_string(&event) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize event: {}", e);
                        continue;
                    }
                };

                let mut sender = sender.lock().await;
                if sender.send(Message::Text(json)).await.is_err() {
                    debug!("Failed to send to client {}", client_id);
                    break;
                }
            }
        };

        tokio::select! {
            _ = recv_task => {},
            _ = send_task => {},
        }

        {
            let mut clients = broadcaster.clients.write().await;
            clients.remove(&client_id);
            info!(
                "WebSocket client disconnected: {} (total: {})",
                client_id,
                clients.len()
            );
        }
    }
}

/// Extractor for getting the broadcaster from app state
#[derive(Debug, Clone)]
pub struct WsState(pub Broadcaster);

impl From<Broadcaster> for WsState {
    fn from(broadcaster: Broadcaster) -> Self {
        Self(broadcaster)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_creation() {
        let broadcaster = Broadcaster::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let count = rt.block_on(async { broadcaster.client_count().await });
        assert_eq!(count, 0);
    }

    #[test]
    fn test_broadcast_event() {
        let broadcaster = Broadcaster::new();
        let event = WsEvent::TaskCreated {
            id: "test-id".to_string(),
            title: "Test Task".to_string(),
        };
        broadcaster.broadcast(event);
    }

    #[test]
    fn test_event_serialization() {
        let event = WsEvent::TaskCreated {
            id: "test-123".to_string(),
            title: "Test Task".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("task_created"));
        assert!(json.contains("test-123"));
        assert!(json.contains("Test Task"));

        let event = WsEvent::TaskUpdated {
            id: "test-456".to_string(),
            status: "IN_PROGRESS".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("task_updated"));
        assert!(json.contains("IN_PROGRESS"));
    }

    #[tokio::test]
    async fn test_client_tracking() {
        let broadcaster = Broadcaster::new();
        assert_eq!(broadcaster.client_count().await, 0);
    }
}
