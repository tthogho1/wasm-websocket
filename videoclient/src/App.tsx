import React, { useEffect, useRef, useState } from 'react';

// wasm-packで生成されたwasm_websocket.jsをimport
import init, { WebSocketClient } from 'pkg/wasm_websocket.js';

const videoStyle: React.CSSProperties = {
  border: '1px solid #333',
  width: '100%',
  minHeight: 240,
  background: '#f0f0f0',
};

const buttonStyle: React.CSSProperties = {
  backgroundColor: '#4CAF50',
  color: 'white',
  border: 'none',
  padding: '8px 16px',
  borderRadius: 4,
  cursor: 'pointer',
};

const inputStyle: React.CSSProperties = {
  flexGrow: 1,
  padding: 8,
  marginRight: 10,
  border: '1px solid #ccc',
  borderRadius: 4,
};

export const App: React.FC = () => {
  const [status, setStatus] = useState<string>('Status: Not connected');
  const [messages, setMessages] = useState<string[]>([]);
  const [messageInput, setMessageInput] = useState<string>('');
  const clientRef = useRef<any>(null);
  const cameraRef = useRef<HTMLVideoElement>(null);
  const remoteRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    (async () => {
      await init();
      const client = new WebSocketClient('ws://localhost:3000');
      clientRef.current = client;

      client.on_open(() => {
        log('Connection opened');
        setStatus('Status: Connected to signaling server');
      });
      client.on_message((msg: string) => {
        log(`Received: ${msg}`);
      });
      client.on_error((error: any) => {
        log(`Error: ${error}`);
        setStatus('Status: Connection error');
      });

      // 画面起動時にカメラ接続
      try {
        setStatus('Status: Accessing camera...');
        const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        if (cameraRef.current) cameraRef.current.srcObject = stream;
        log('Webcam started');
        setStatus('Status: Camera access successful');
        // Rust側で自動的にadd_media_streamされる想定
      } catch (error: any) {
        log(`Error accessing the webcam: ${error.message}`);
        setStatus('Status: Camera access failed');
      }
    })();
    // eslint-disable-next-line
  }, []);

  function log(message: string) {
    console.log(message);
    setMessages(msgs => [...msgs, message]);
  }

  function handleSend() {
    if (!clientRef.current) return;
    clientRef.current.send_message(messageInput);
    log(`Sent: ${messageInput}`);
    setMessageInput('');
  }

  async function handleWebcamOffer() {
    try {
      setStatus('Status: Accessing camera...');
      const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
      if (cameraRef.current) cameraRef.current.srcObject = stream;
      log('Webcam started');
      setStatus('Status: Camera access successful');
      if (clientRef.current) {
        log('Sending WebRTC offer...');
        await clientRef.current.offer();
        log('Connection offer sent');
        setStatus('Status: WebRTC offer sent, waiting for answer...');
      }
    } catch (error: any) {
      log(`Error accessing the webcam: ${error.message}`);
      setStatus('Status: Camera access failed');
    }
  }

  return (
    <div style={{ fontFamily: 'Arial, sans-serif', maxWidth: 800, margin: '0 auto', padding: 20 }}>
      <h1>WebRTC Video Chat</h1>
      <div className="status" style={{ color: '#666', fontStyle: 'italic', marginBottom: 10 }}>
        {status}
      </div>
      <div
        className="video-container"
        style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 20 }}
      >
        <div className="video-box" style={{ width: '48%' }}>
          <h2>Local Video</h2>
          <video id="camera" ref={cameraRef} autoPlay muted playsInline style={videoStyle} />
        </div>
        <div className="video-box" style={{ width: '48%' }}>
          <h2>Remote Video</h2>
          <video id="remoteVideo" ref={remoteRef} autoPlay playsInline style={videoStyle} />
        </div>
      </div>
      <div className="controls" style={{ margin: '20px 0', display: 'flex', gap: 10 }}>
        <button id="webcamButton" onClick={handleWebcamOffer} style={buttonStyle}>
          Start Webcam &amp; Connect
        </button>
      </div>
      <div
        id="messages"
        style={{
          border: '1px solid #ccc',
          height: 300,
          overflowY: 'scroll',
          padding: 10,
          marginBottom: 10,
        }}
      >
        {messages.map((msg, i) => (
          <div key={i}>{msg}</div>
        ))}
      </div>
      <div className="message-input" style={{ display: 'flex', marginTop: 10 }}>
        <input
          type="text"
          id="messageInput"
          placeholder="Enter message"
          value={messageInput}
          onChange={e => setMessageInput(e.target.value)}
          style={inputStyle}
        />
        <button id="sendButton" onClick={handleSend} style={buttonStyle}>
          Send
        </button>
      </div>
    </div>
  );
};
