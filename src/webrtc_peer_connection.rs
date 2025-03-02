use wasm_bindgen::prelude::*;
use web_sys::{RtcPeerConnection  ,RtcConfiguration,RtcPeerConnectionIceEvent, RtcSessionDescriptionInit, RtcIceCandidateInit};
use js_sys::{Object, Reflect};

#[wasm_bindgen]
pub struct WebRTCConnection {
    peer_connection: RtcPeerConnection,
}

#[wasm_bindgen]
impl WebRTCConnection {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WebRTCConnection, JsValue> {
        // RTCPeerConnection設定
        //let config = Object::new();
        // Create an RtcConfiguration object
        let mut config = RtcConfiguration::new();

        Reflect::set(&config, &"iceServers".into(), 
            &js_sys::Array::of1(&get_ice_server()))?;

        let peer_connection = RtcPeerConnection::new_with_configuration(&config)?;
        //let peer_connection = RtcPeerConnection::new()?;
        // ICEイベントリスナーの設定
        let ice_candidate_closure = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = event.candidate() {
                console_log(&format!("ICE Candidate: {:?}", candidate));
                // ここでcandidateをシグナリングサーバーに送信
            }
        }) as Box<dyn Fn(RtcPeerConnectionIceEvent)>);

        peer_connection.set_onicecandidate(Some(ice_candidate_closure.as_ref().unchecked_ref()));
        ice_candidate_closure.forget();

        Ok(WebRTCConnection { peer_connection })
    }

    // オファー生成
    pub async fn create_offer(&self) -> Result<JsValue, JsValue> {
        let promise = self.peer_connection.create_offer();
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
