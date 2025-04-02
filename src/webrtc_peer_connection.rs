use wasm_bindgen::prelude::*;
use web_sys::{RtcPeerConnection  ,RtcConfiguration,RtcPeerConnectionIceEvent, RtcSessionDescriptionInit, RtcIceCandidateInit};
use js_sys::{Object, Reflect};

#[wasm_bindgen]
#[derive(Clone)]    
pub struct WebRTCConnection {
    peer_connection: RtcPeerConnection,
}

#[wasm_bindgen]
impl WebRTCConnection {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WebRTCConnection, JsValue> {
        // RTCPeerConnection設定
        // Create an RtcConfiguration object
        console_log(&format!("start webrtc connection"));
        let config = RtcConfiguration::new();

        Reflect::set(&config, &"iceServers".into(), &js_sys::Array::of1(&get_ice_server()))?;

        let peer_connection = RtcPeerConnection::new_with_configuration(&config)?;
        // ICEイベントリスナーの設定
        let ice_candidate_closure = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = event.candidate() {
                console_log(&format!("ICE Candidate: {:?}", candidate));
                // ここでcandidateをシグナリングサーバーに送信
            }
        }) as Box<dyn Fn(RtcPeerConnectionIceEvent)>);

        peer_connection.set_onicecandidate(Some(ice_candidate_closure.as_ref().unchecked_ref()));
        ice_candidate_closure.forget();


        // ICE接続状態の変更を監視するイベントハンドラーを設定
        let on_ice_connection_state_change = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
            let connection_state = Reflect::get(&event, &"target".into())
                .and_then(|target| Reflect::get(&target, &"iceConnectionState".into()))
                .unwrap_or_else(|_| JsValue::from("unknown"));
    
            web_sys::console::log_1(&format!("ICE connection state changed: {:?}", connection_state).into());
        }) as Box<dyn FnMut(_)>);

        peer_connection.set_oniceconnectionstatechange(Some(on_ice_connection_state_change.as_ref().unchecked_ref()));
        on_ice_connection_state_change.forget(); // メモリリークを防ぐためにClosureを保持
    
        let peer_connection_clone = peer_connection.clone();
        let on_signaling_state_change = Closure::wrap(Box::new(move || {
            let signaling_state = Reflect::get(&peer_connection_clone, &JsValue::from_str("signalingState"))
                .unwrap_or_else(|_| JsValue::from_str("unknown"));
    
            web_sys::console::log_1(&format!("Signaling state changed: {:?}", signaling_state).into());
        }) as Box<dyn FnMut()>);
    
        peer_connection.set_onsignalingstatechange(Some(on_signaling_state_change.as_ref().unchecked_ref()));
        on_signaling_state_change.forget(); // メモリリークを防ぐためにClosureを保持


        Ok(WebRTCConnection { peer_connection })
    }

    // オファー生成
    pub async fn create_offer(&self) -> Result<JsValue, JsValue> {
        let promise = self.peer_connection.create_offer();
        wasm_bindgen_futures::JsFuture::from(promise).await
    }

    pub async fn create_answer(&self) -> Result<JsValue, JsValue> {
        let promise = self.peer_connection.create_answer();
        wasm_bindgen_futures::JsFuture::from(promise).await
    }

    // セッション記述の設定
    pub async fn set_local_description(&self, description: &RtcSessionDescriptionInit) -> Result<(), JsValue> {
        let promise = self.peer_connection.set_local_description(description);
        wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(())
    }

    // リモート記述の設定
    pub async fn set_remote_description(&self, description: &RtcSessionDescriptionInit) -> Result<(), JsValue> {
        let promise = self.peer_connection.set_remote_description(description);
        wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(())
    }

    // ICE Candidateの追加
    pub async fn add_ice_candidate(&self, candidate: &RtcIceCandidateInit) -> Result<(), JsValue> {
        let promise = self.peer_connection.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(candidate));
        wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(())
    }
}

// ICEサーバー設定のヘルパー関数
fn get_ice_server() -> Object {
    let ice_server = Object::new();
    Reflect::set(&ice_server, &"urls".into(), &"stun:stun.l.google.com:19302".into()).unwrap();
    ice_server
}

// コンソールログのヘルパー関数
fn console_log(message: &str) {
    web_sys::console::log_1(&message.into());
}
