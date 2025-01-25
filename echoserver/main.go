package main

import (
	"log"
	"net/http"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

func handleConnections(w http.ResponseWriter, r *http.Request) {
	// HTTP接続をWebSocket接続にアップグレード
	log.Println("WebSocket connection opened.")
	upgrader.CheckOrigin = func(r *http.Request) bool {
		// Originヘッダーをチェック
		return r.Header.Get("Origin") == "http://localhost"
	}
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		return
	}
	defer conn.Close()

	// メッセージ受信ループ
	for {
		messageType, p, err := conn.ReadMessage()
		if err != nil {
			log.Println(err)
			return
		}

		log.Printf("Received message: %s", p)
		// 受信したメッセージをそのまま返信
		if err := conn.WriteMessage(messageType, p); err != nil {
			log.Println(err)
			return
		}
	}
}

func main() {
	http.HandleFunc("/ws", handleConnections)
	log.Println("Listening on 8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
