use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::connect_async;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use std::sync::{Arc, Mutex};

type SharedWebSocket = Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>;

pub struct WebSocketManager {
    ws_sink: SharedWebSocket,
    url: String, // Store the URL in the struct
}

impl WebSocketManager {
    pub fn new(url: String) -> Self {
        WebSocketManager {
            ws_sink: Arc::new(Mutex::new(None)),
            url, // Initialize the URL field
        }
    }

    pub async fn connect(&self) {
        if let Ok((ws_stream, _)) = connect_async(&self.url).await {
            // Split the WebSocket stream
            let (ws_sink, _) = ws_stream.split();
            let mut ws_sink_lock = self.ws_sink.lock().unwrap();
            *ws_sink_lock = Some(ws_sink);
            println!("Connected to WebSocket server at {}", self.url);
        } else {
            eprintln!("Failed to connect to WebSocket server at {}", self.url);
        }
    }

    pub async fn send_message(&self, message: &str) {
        let mut ws_sink_lock = self.ws_sink.lock().unwrap();
        if let Some(ws_sink) = &mut *ws_sink_lock {
            if let Err(e) = ws_sink.send(Message::Text(message.to_string())).await {
                eprintln!("Failed to send message: {}", e);
            }
        } else {
            eprintln!("WebSocket connection not established");
        }
    }
}
