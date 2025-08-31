#[derive(Clone, Debug)]
pub enum MessagePayload {
    Bytes12([u8; 12]),
    Bytes24([u8; 24]),
    Bytes48([u8; 48]),
    Bytes64([u8; 64]),
    Bytes128([u8; 128]),
    Large(Box<[u8]>),
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: &'static str,
    pub payload: MessagePayload,
    pub len: usize,
}

impl Message {
    pub fn new<T: serde::Serialize>(id: &'static str, data: &T) -> Self {
        let bytes = bincode::serialize(data).unwrap();
        
        let payload = match bytes.len() {

            0..=12 => {
                let mut arr = [0u8; 12];
                arr[..bytes.len()].copy_from_slice(&bytes);
                MessagePayload::Bytes12(arr)
            }
            13..=24 => {
                let mut arr = [0u8; 24];
                arr[..bytes.len()].copy_from_slice(&bytes);
                MessagePayload::Bytes24(arr)
            }
            25..=48 => {
                let mut arr = [0u8; 48];
                arr[..bytes.len()].copy_from_slice(&bytes);
                MessagePayload::Bytes48(arr)
            }
            49..=64 => {
                let mut arr = [0u8; 64];
                arr[..bytes.len()].copy_from_slice(&bytes);
                MessagePayload::Bytes64(arr)
            }
            65..=128 => {
                let mut arr = [0u8; 128];
                arr[..bytes.len()].copy_from_slice(&bytes);
                MessagePayload::Bytes128(arr)
            }
            _ => MessagePayload::Large(bytes.into_boxed_slice()),
        };

        Self { id, payload }
    }
    
    pub fn serialized_data(&self) -> &[u8] {
        match &self.payload {
            MessagePayload::Bytes12(arr) => arr,
            MessagePayload::Bytes24(arr) => arr,
            MessagePayload::Bytes48(arr) => arr,
            MessagePayload::Bytes64(arr) => arr,
            MessagePayload::Bytes128(arr) => arr,
            MessagePayload::Large(boxed) => boxed,
        }
    }
}

#[async_trait::async_trait]
pub trait SystemHandler: Send + Sync {
    async fn run(&self, mut inbox: mpsc::UnboundedReceiver<Message>);
}

pub struct Spine {
    system_channels: Arc<ArcSwap<HashMap<String, mpsc::UnboundedSender<Message>>>>, // Arc because
                                                                                    // it needs to
                                                                                    // be moved
                                                                                    // across
                                                                                    // threads,
                                                                                    // ArcSwap for
                                                                                    // hot-swappable
                                                                                    // mods
    message_router: Arc<ArcSwap<HashMap<String, Vec<String>>>>, // Same thing as on top
}

impl Spine {
    pub fn register_system(&mut self, system_name: String, handler: Box<dyn SystemHandler>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.system_channels.insert(system_name, tx);
        
        tokio::spawn(async move {
            handler.run(rx).await;
        });
    }
    
    pub fn publish(&self, message: Message) {
        let router = self.message_router.load();
        
        if let Some(subscribers) = router.get(&message.id) {
            for system_name in subscribers {
                if let Some(sender) = self.system_channels.get(system_name) {
                    if let Err(_) = sender.send(message.clone()) { // Deep clones the message only
                                                                   // when message payload is
                                                                   // Large, otherwise its
                                                                   // stack-allocation
                        // System channel is closed, maybe remove it from routing?
                    }
                }
            }
        }
    }
}
