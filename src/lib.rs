use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{WebSocket, MessageEvent, ErrorEvent,console, RtcSessionDescriptionInit,RtcSdpType };
mod webrtc_peer_connection;
use webrtc_peer_connection::WebRTCConnection;
use js_sys::JsString;
use serde_json;


#[wasm_bindgen]
pub struct WebSocketClient {
    peerconnection: Option<WebRTCConnection>,
    ws: WebSocket,
}

#[wasm_bindgen]
impl WebSocketClient {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<WebSocketClient, JsValue> {
        // Create WebSocket connection
        let formatted_log = format!("url: {}", url);
        console::log_1(&formatted_log.into());

        // craete webrtc peerconnection
        let peer = Some(WebRTCConnection::new().unwrap());
        console::log_1(&"WebRtc connection create.".into());
        // create websocket
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

        // 
        Ok(WebSocketClient {peerconnection: peer, ws })
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
            console::log_1(&format!("Received message from WebSocket: {:?}", message.as_string()).into());
            // judge if message is json or not
            if let Some(text) = message.as_string() {
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        console::log_1(&format!("set Offer JSON : {:?}", json).into());
                        // set offer to peerconnection 
                        let mut offer = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
                        offer.set_sdp(json["sdp"].as_str().unwrap());

                        self.peerconnection.as_ref().unwrap().set_remote_description(&offer);
                        console::log_1(&"Set offer to peerconnection.".into());
                        // send answer to websocket
                        let answer_promise = self.peerconnection.as_ref().unwrap().create_answer();
                        console::log_1(&"Created answer promise.".into());
                        
                        let _ = answer_promise.then(&Closure::once_into_js(move |answer_jsval| {
                            console::log_1(&"Answer promise resolved.".into());
                            let answer_str = js_sys::JSON::stringify(&answer_jsval).unwrap_or_else(|_| JsString::from(""));
                            if let Some(sdp_str) = answer_str.as_string() {
                                self.ws.send_with_str(&sdp_str).unwrap();
                            } else {
                                console::log_1(&JsValue::from_str("Failed to extract SDP"));
                            }
                        }));

                    },
                    Err(_) => {
                        console::log_1(&format!("Received non-JSON message: {}", text).into());
                        let _ = callback.call1(&cloned_ws, &message);
                    }
                }
            } else {
                println!("Received non-text message");
            }
            // let _ = callback.call1(&cloned_ws, &message);
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

    pub async fn offer(&mut self) -> () {
        let offer = self.peerconnection.as_ref().unwrap().create_offer().await.unwrap();
        
        let offer_str = js_sys::JSON::stringify(&offer).unwrap_or_else(|_| JsString::from(""));
        if let Some(sdp_str) = offer_str.as_string() {
            self.ws.send_with_str(&sdp_str).unwrap();
        } else {
            console::log_1(&JsValue::from_str("Failed to extract SDP"));
        }
    }

}