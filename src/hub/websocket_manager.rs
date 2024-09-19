use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, info};

type SharedWebSocket =
    Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>;

pub struct WebSocketManager {
    ws_sink: SharedWebSocket,
    url: String,
}

impl WebSocketManager {
    pub fn new(url: String) -> Self {
        WebSocketManager {
            ws_sink: Arc::new(Mutex::new(None)),
            url,
        }
    }

    pub async fn connect(&self) {
        if let Ok((ws_stream, _)) = connect_async(&self.url).await {
            let (ws_sink, _) = ws_stream.split();
            let mut ws_sink_lock = self.ws_sink.lock().unwrap();
            *ws_sink_lock = Some(ws_sink);
            info!("Connected to WebSocket server at {}", self.url);
        } else {
            error!("Failed to connect to WebSocket server at {}", self.url);
        }
    }

    pub async fn send_message(&self, message: &str) {
        let mut ws_sink_lock = self.ws_sink.lock().unwrap();
        if let Some(ws_sink) = &mut *ws_sink_lock {
            if let Err(e) = ws_sink.send(Message::Text(message.to_string())).await {
                error!("Failed to send message: {}", e);
            }
        } else {
            error!("WebSocket connection not established");
        }
    }
}
