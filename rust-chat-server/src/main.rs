use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};
use uuid::Uuid;

type Clients = Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    info!("Server is listening on http://{}", addr);

    // Store connected clients
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    while let Ok((stream, _)) = listener.accept().await {
        let clients = Arc::clone(&clients);
        tokio::spawn(handle_connection(stream, clients));
    }
}

async fn handle_connection(stream: TcpStream, clients: Clients) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    let client_id = Uuid::new_v4().to_string();
    info!("New client connected: {}", client_id);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    clients.write().await.insert(client_id.clone(), tx);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Handle outgoing messages
    let client_id_clone = client_id.clone();
    let clients_clone = Arc::clone(&clients);
    let send_task = tokio::spawn(async move {
        let mut rx = rx;
        while let Some(message) = rx.recv().await {
            if ws_sender.send(message).await.is_err() {
                break;
            }
        }
        // Remove client when send task ends
        clients_clone.write().await.remove(&client_id_clone);
    });

    // Handle incoming messages
    let client_id_clone = client_id.clone();
    let clients_clone = Arc::clone(&clients);
    let receive_task = tokio::spawn(async move {
        while let Some(message) = ws_receiver.next().await {
            match message {
                Ok(msg) => {
                    let decoded_string = decode_message(msg).await;
                    info!("Received message from {}: {}", client_id_clone, decoded_string);
                    
                    // Broadcast to all other clients
                    broadcast_message(&clients_clone, &client_id_clone, decoded_string).await;
                }
                Err(e) => {
                    warn!("Error receiving message from {}: {}", client_id_clone, e);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    // Clean up
    clients.write().await.remove(&client_id);
    info!("Client disconnected: {}", client_id);
}

async fn decode_message(message: Message) -> String {
    match message {
        Message::Text(text) => {
            info!("Received string message: {}", text);
            text
        }
        Message::Binary(data) => {
            info!("Received binary message");
            match String::from_utf8(data) {
                Ok(decoded) => {
                    info!("Decoded binary message: {}", decoded);
                    decoded
                }
                Err(e) => {
                    warn!("Failed to decode binary message as UTF-8: {}", e);
                    String::new()
                }
            }
        }
        Message::Close(_) => {
            info!("Received close message");
            String::new()
        }
        Message::Ping(_) => {
            info!("Received ping message");
            String::new()
        }
        Message::Pong(_) => {
            info!("Received pong message");
            String::new()
        }
        Message::Frame(_) => {
            warn!("Received raw frame message");
            String::new()
        }
    }
}

async fn broadcast_message(clients: &Clients, sender_id: &str, message: String) {
    let clients_guard = clients.read().await;
    let message_to_send = Message::Text(message);
    
    for (client_id, sender) in clients_guard.iter() {
        // Send message to all clients except the sender
        if client_id != sender_id {
            if let Err(e) = sender.send(message_to_send.clone()) {
                warn!("Failed to send message to client {}: {}", client_id, e);
            }
        }
    }
}
