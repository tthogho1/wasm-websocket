import express from 'express';
import { Server } from 'ws';
import WebSocket from 'ws';

const app = express();
const port = 3000;

// Create HTTP server
const server = app.listen(port, () => {
    console.log(`Server is listening on http://localhost:${port}`);
});


const decodeMessage = async (message :any):Promise<string> => {
    let decodedString = "";

    if (typeof message === 'string') {
        console.log(`Received string message: ${message}`);
        decodedString = message;
    } else if (message instanceof ArrayBuffer) {
        console.log('Received binary message');
        const uint8Array = new Uint8Array(message);
        const decoder = new TextDecoder('utf-8');
        
        decodedString = decoder.decode(uint8Array);
    } else if (message instanceof Blob){
        console.log("Received blob message"); 
        decodedString = await message.text();
    } else if (message instanceof Uint8Array){
        console.log(`Received Unit8Array ${message}`);
        const decoder = new TextDecoder("utf-8");

        decodedString = decoder.decode(message);
        console.log("Received Unit8Array message: " + decodedString);
    }else {
        console.log('Received unknown message type');
    }

    return decodedString;
}


// Create WebSocket server
const wss = new Server({ server });

wss.on('connection', (ws) => {
    console.log('New client connected');

    ws.on('message', async (message) => {
        console.log(`Received message of type: ${typeof message}`);
        console.log(`Detailed type: ${Object.prototype.toString.call(message)}`);

        const decodedString = await decodeMessage(message);
        wss.clients.forEach((client) => {
            // Send message to all clients except the sender
            if (client !== ws && client.readyState === WebSocket.OPEN) {
                client.send(decodedString);
            }
        });

    });

    ws.on('close', () => {
        console.log('Client disconnected');
    });
});