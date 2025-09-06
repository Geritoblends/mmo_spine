use tokio::sync::{broadcast, mpsc};

#[derive(Clone, Copy)]
pub struct Message {
    id: [u8; 32],
    payload: [u8; 255],
    payload_len: u8,
}

pub struct Handler {
    id: [u8; 32],
    command_receiver: mpsc::Receiver<Message>,
    command_senders: ArcSwap<HashMap<[u8; 32], mpsc::Sender<Message>>>,
    message_receivers: Vec<broadcast::Receiver<Message>>,
    message_sender: broadcast::Sender<Message>
}

// Pending
// 1. Find a way to serialize arbitrary types into either a message or an array of messages (or a
//    Vec::with_capacity) ergonomically
// 2. Ergonomics layer
// 3. Message pooling system
// 4. Handler discovery/routing
// 5. Deterministic ID generation (4.)
// 6. proof with simple handlers
//
