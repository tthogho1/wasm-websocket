use wasm_bindgen::prelude::*;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, console};
mod webrtc_peer_connection;
use webrtc_peer_connection::WebRTCConnection;
use js_sys::JsString;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[wasm_bindgen]
pub struct WebSocketClient {
    ws: WebSocket,
}

#[wasm_bindgen]
impl WebSocketClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<WebSocketClient, JsValue> {
        // Create WebSocket connection

        let formatted_log = format!("url: {}", url);
        console::log_1(&formatted_log.into());
        let ws = match WebSocket::new(url) {
            Ok(socket) => { 
                console::log_1(&"WebSocket connection create.".into());
                socket 
            } 
            Err(err) => {
                console::log_1(&format!("Failed to connect to WebSocket: {:?}", err).into());
                return Err(err);
            }
        };
        console::log_1(&"WebSocket connection create.".into());

        // Set binary type to arraybuffer
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        console::log_1(&"WebSocket set binary type.".into());

        Ok(WebSocketClient { ws })
    }

    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        console::log_1(&format!("Sending message to WebSocket: {:?}", message).into());

        self.ws.send_with_str(message)
    }

    pub fn on_open(&self, callback: js_sys::Function) -> Result<(), JsValue> {
        console::log_1(&"WebSocket on open.".into());

        let cloned_ws = self.ws.clone();
        let closure = Closure::wrap(Box::new(move |_event: JsValue| {
            let _ = callback.call0(&cloned_ws);
        }) as Box<dyn Fn(JsValue)>);

        self.ws.set_onopen(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        Ok(())
    }

    pub fn on_message(&self, callback: js_sys::Function) -> Result<(), JsValue> {
        console::log_1(&"WebSocket on message.".into());
        let cloned_ws = self.ws.clone();
        let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
            let message = event.data();



            let _ = callback.call1(&cloned_ws, &message);
        }) as Box<dyn Fn(MessageEvent)>);

        self.ws.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        Ok(())
    }

    pub fn on_error(&self, callback: js_sys::Function) -> Result<(), JsValue> {
        let cloned_ws = self.ws.clone();
        let closure = Closure::wrap(Box::new(move |event: ErrorEvent| {
            let _ = callback.call1(&cloned_ws, &event);
        }) as Box<dyn Fn(ErrorEvent)>);

        self.ws.set_onerror(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        Ok(())
    }

    pub fn close(&self) -> Result<(), JsValue> {
        self.ws.close()
    }

    pub async fn offer(&self) -> () {
        let connection = WebRTCConnection::new().unwrap();
        let offer = connection.create_offer().await.unwrap();
        
        let offer_str = js_sys::JSON::stringify(&offer).unwrap_or_else(|_| JsString::from(""));
        if let Some(sdp_str) = offer_str.as_string() {
            self.ws.send_with_str(&sdp_str).unwrap();
        } else {
            console::log_1(&JsValue::from_str("Failed to extract SDP"));
        }
    }

}