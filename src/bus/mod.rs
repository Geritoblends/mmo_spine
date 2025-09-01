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
    system_channels: Arc<HashMap<&'static str, mpsc::UnboundedSender<Message>>>, 
    message_router: Arc<HashMap<&'static str, Vec<&'static str>>>, 
}

impl Spine {

pub fn new(systems: Vec<(&'static str, Box<dyn SystemHandler>, Vec<&'static str>)>) -> Self {
        let mut system_channels = HashMap::new();
        let mut message_router = HashMap::new();
        
        for (system_name, handler, subscriptions) in systems {
            let (tx, rx) = mpsc::unbounded_channel();
            system_channels.insert(system_name, tx);
            for message_id in subscriptions {
                message_router
                    .entry(message_id)
                    .or_insert_with(Vec::new)
                    .push(system_name);
            }

            tokio::spawn(async move {
                handler.run(rx).await;
            });
        }
        
        Self {
            system_channels: Arc::new(system_channels),
            message_router: Arc::new(message_router),
        }
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
