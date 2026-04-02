use base64::{Engine as _, engine::general_purpose::STANDARD};
use claude_rust_errors::{AppError, AppResult};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::domain::bridge_types::BridgeMessage;

pub struct WebSocketTransport;

impl WebSocketTransport {
    pub fn new() -> Self {
        Self
    }

    pub async fn accept_connection(&self, mut stream: TcpStream) -> AppResult<BridgeConnection> {
        // Read the HTTP upgrade request
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let request = String::from_utf8_lossy(&buf[..n]);

        // Extract Sec-WebSocket-Key
        let key = request.lines()
            .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
            .and_then(|l| l.splitn(2, ':').nth(1))
            .map(|s| s.trim())
            .ok_or_else(|| AppError::BadRequest("missing Sec-WebSocket-Key".into()))?;

        let accept = compute_accept_key(key);

        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\
             \r\n",
            accept
        );

        stream.write_all(response.as_bytes()).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        Ok(BridgeConnection { stream })
    }
}

impl Default for WebSocketTransport {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_accept_key(key: &str) -> String {
    const MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let input = format!("{}{}", key, MAGIC);
    let hash = sha1_bytes(input.as_bytes());
    STANDARD.encode(hash)
}

fn sha1_bytes(data: &[u8]) -> [u8; 20] {
    // SHA-1 implementation
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let bit_len = (data.len() as u64) * 8;

    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0x00);
    }
    for i in (0..8).rev() {
        msg.push((bit_len >> (i * 8)) as u8);
    }

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);

        for i in 0..80 {
            let (f, k) = match i {
                0..=19  => ((b & c) | ((!b) & d),          0x5A827999u32),
                20..=39 => (b ^ c ^ d,                      0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d),   0x8F1BBCDCu32),
                _       => (b ^ c ^ d,                      0xCA62C1D6u32),
            };
            let temp = a.rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    let mut out = [0u8; 20];
    for (i, &val) in h.iter().enumerate() {
        out[i*4..i*4+4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

pub struct BridgeConnection {
    stream: TcpStream,
}

impl BridgeConnection {
    pub async fn send(&mut self, msg: BridgeMessage) -> AppResult<()> {
        let payload = serde_json::to_vec(&msg)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let frame = encode_websocket_frame(&payload);
        self.stream.write_all(&frame).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Option<BridgeMessage> {
        let payload = decode_websocket_frame(&mut self.stream).await?;
        serde_json::from_slice(&payload).ok()
    }
}

fn encode_websocket_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    // FIN=1, opcode=1 (text)
    frame.push(0x81);
    let len = payload.len();
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.push((len >> 8) as u8);
        frame.push(len as u8);
    } else {
        frame.push(127);
        for i in (0..8).rev() {
            frame.push((len >> (i * 8)) as u8);
        }
    }
    frame.extend_from_slice(payload);
    frame
}

async fn decode_websocket_frame(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await.ok()?;

    let _fin = (header[0] & 0x80) != 0;
    let opcode = header[0] & 0x0F;
    // opcode 8 = close
    if opcode == 8 {
        return None;
    }

    let masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as usize;

    if payload_len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).await.ok()?;
        payload_len = u16::from_be_bytes(ext) as usize;
    } else if payload_len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).await.ok()?;
        payload_len = u64::from_be_bytes(ext) as usize;
    }

    let mask = if masked {
        let mut m = [0u8; 4];
        stream.read_exact(&mut m).await.ok()?;
        Some(m)
    } else {
        None
    };

    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload).await.ok()?;

    if let Some(mask) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    Some(payload)
}
