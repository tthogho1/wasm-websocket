# Rust Chat Server

TypeScript の`server.ts`と同じ機能を持つ Rust 製 WebSocket サーバーです。

## 機能

- WebSocket 接続の受け入れ
- メッセージの受信（テキスト・バイナリ対応）
- 送信者以外の全クライアントへのブロードキャスト
- クライアント接続・切断の管理

## 実行方法

```bash
cargo run
```

サーバーは `http://127.0.0.1:3000` で起動します。

## 依存関係

- `tokio`: 非同期ランタイム
- `tokio-tungstenite`: WebSocket サーバー実装
- `futures-util`: 非同期ストリーム処理
- `uuid`: クライアント ID 生成
- `tracing`: ログ出力
- `serde`: JSON 処理

## TypeScript 版との違い

- Rust 版では各クライアントに一意の UUID を割り当て
- 非同期タスクで送受信を並行処理
- メモリ安全性とパフォーマンスの向上
