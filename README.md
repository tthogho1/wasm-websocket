# WebSocket Client with Wasm

## echoserver Folder
This folder contains an echo server implemented in Go. It simply returns the messages received from WebSocket clients.

## pythonclient Folder
This folder includes a Python client script that checks the connection with the WebSocket server.

## src (Wasm Program)
Build the Wasm program using the following command:
```bash
wasm-pack build --target web
```
This command generates the module in the `pkg` folder.

## index.html
Place the `index.html` file alongside the `pkg` folder on the web server.