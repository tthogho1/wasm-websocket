use wasm_bindgen::prelude::*;
use web_sys::{ RtcPeerConnection, RtcConfiguration, RtcPeerConnectionIceEvent, RtcSessionDescriptionInit, RtcIceCandidateInit, HtmlVideoElement, MediaStream, MediaStreamConstraints, Document, Window, RtcTrackEvent};
use js_sys::{Object, Reflect};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;


#[wasm_bindgen]
#[derive(Clone)]    
pub struct WebRTCConnection {
    peer_connection: RtcPeerConnection,
    ws: web_sys::WebSocket,
}

#[wasm_bindgen]
impl WebRTCConnection{
    #[wasm_bindgen(constructor)]
    pub fn new(ws: web_sys::WebSocket) -> Result<WebRTCConnection, JsValue> {
        // RTCPeerConnection設定
        // Create an RtcConfiguration object
        console_log(&format!("start webrtc connection"));
        let config = RtcConfiguration::new();

        Reflect::set(&config, &"iceServers".into(), &js_sys::Array::of1(&get_ice_server()))?;

        let peer_connection = RtcPeerConnection::new_with_configuration(&config)?;
        // ICEイベントリスナーの設定
        // let peer_connection_clone = peer_connection.clone();
        let ws_sender = ws.clone(); // WebSocketをクロージャに渡す
        let ice_candidate_closure = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = event.candidate() {
                console_log(&format!("ICE Candidate: {:?}", candidate));
                // candidateをJSON文字列にしてWebSocketで送信
                let candidate_json = js_sys::Object::new();
                js_sys::Reflect::set(&candidate_json, &JsValue::from_str("type"), &JsValue::from_str("icecandidate")).unwrap();
                js_sys::Reflect::set(&candidate_json, &JsValue::from_str("candidate"), &candidate).unwrap();
                let json_str = js_sys::JSON::stringify(&candidate_json).unwrap();
                ws_sender.send_with_str(&json_str.as_string().unwrap()).unwrap();
            }
        }) as Box<dyn Fn(RtcPeerConnectionIceEvent)>);

        peer_connection.set_onicecandidate(Some(ice_candidate_closure.as_ref().unchecked_ref()));
        ice_candidate_closure.forget();


        // ICE接続状態の変更を監視するイベントハンドラーを設定
        let on_ice_connection_state_change = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
            let connection_state = Reflect::get(&event, &"target".into())
                .and_then(|target| Reflect::get(&target, &"iceConnectionState".into()))
                .unwrap_or(JsValue::from("unknown"));
            
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

        // Add ontrack event handler to handle incoming media tracks
        let on_track = Closure::wrap(Box::new(move |event: RtcTrackEvent| {
            web_sys::console::log_1(&"Received remote track".into());

            let streams = event.streams();
            if streams.length() == 0 {
                return;
            }
            let remote_stream = streams.get(0);
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };
            let document = match window.document() {
                Some(d) => d,
                None => return,
            };
            let video_element = match document.get_element_by_id("remoteVideo") {
                Some(e) => e,
                None => return,
            };
            let video = match video_element.dyn_into::<HtmlVideoElement>() {
                Ok(v) => v,
                Err(_) => return,
            };
            let media_stream = match remote_stream.dyn_into::<MediaStream>() {
                Ok(m) => m,
                Err(_) => {
                    web_sys::console::log_1(&"Failed to cast remote stream to MediaStream".into());
                    return;
                }
            };
            video.set_src_object(Some(&media_stream));
            web_sys::console::log_1(&"Remote video stream connected".into());
            if let Some(status_element) = document.get_element_by_id("connectionStatus") {
                let _ = status_element.set_text_content(Some("Status: Remote video connected"));
            }
        }) as Box<dyn FnMut(RtcTrackEvent)>);

        peer_connection.set_ontrack(Some(on_track.as_ref().unchecked_ref()));
        on_track.forget();
        Ok(WebRTCConnection { peer_connection, ws })
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

    pub fn add_media_stream(&self, stream: &MediaStream) -> Result<(), JsValue> {
        let tracks = stream.get_tracks();
        console_log(&format!("Adding {} tracks to peer connection", tracks.length()));
        self.peer_connection.add_stream(stream); 
        // for i in 0..tracks.length() {
        //     let track = tracks.get(i).unchecked_into();
        //     // ストリームの配列を作成
        //     // let streams = js_sys::Array::new();
        //     // streams.push(stream);
        //     // // streamを渡す。
        //     let streams = js_sys::Array::of1(stream);
        //     self.peer_connection.add_stream(stream); .add_track(&track, &streams)?;
        // }
        Ok(())
    }
}

// #[wasm_bindgen(start)]
// pub fn start() {
//     wasm_bindgen_futures::spawn_local(async {
//         if let Err(e) = start_camera().await {
//             web_sys::console::error_1(&e);
//         }
//     });
// }


pub async fn start_camera(peer: WebRTCConnection) -> Result<(), JsValue> {
    // videoタグを取得
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let video = document
        .get_element_by_id("camera")
        .unwrap()
        .dyn_into::<HtmlVideoElement>()?;

    // カメラの制約を設定
    let mut constraints = MediaStreamConstraints::new();
    constraints.video(&JsValue::TRUE);
    constraints.audio(&JsValue::FALSE);

    // getUserMediaを呼び出す
    let media_devices = window.navigator().media_devices()?;
    let media_promise = media_devices.get_user_media_with_constraints(&constraints)?;

    // PromiseをFutureに変換してawait
    let stream = JsFuture::from(media_promise).await?;

    // MediaStreamに変換してvideoタグにセット
    let media_stream = stream.dyn_into::<MediaStream>()?;
    video.set_src_object(Some(&media_stream));

    peer.add_media_stream(&media_stream)?;

    Ok(())
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
