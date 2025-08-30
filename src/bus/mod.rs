pub enum MessagePayload {
    PlayerPositionUpdate { player_id: u64, x: f32, y: f32, z: f32 },
    PlayerHealthChange { player_id: u64, delta: i32 },
    ItemPickup { player_id: u64, item_id: u64 },
    ChatMessage { player_id: u64, text: String },
    Custom {
        type_name: &'static str,
        payload: Box<dyn Any + Send + Sync>,
    }
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: &'static str,
    payload: MessagePayload,
}

// Mods would define message types like this:
const PLAYER_ATTACK: &'static str = "player.attack";
const SPELL_CAST: &'static str = "magic.spell_cast";

// Super fast string comparisons (just pointer equality, not string content)
if message.id == PLAYER_ATTACK {  // Compares pointers, not characters!
    // handle attack
}

impl Message {
    fn get_id(&self) -> &'a str
}

#[async_trait::async_trait]
pub trait SystemHandler: Send + Sync {
    async fn run(&self, mut inbox: mpsc::UnboundedReceiver<Message>);
}

pub struct MMOSpine {
    system_channels: HashMap<String, mpsc::UnboundedSender<Message>>,
    message_router: Arc<ArcSwap<HashMap<String, Vec<String>>>>,
    state_store: Arc<StateStore>,
}

impl MMOSpine {
    pub fn register_system(&mut self, system_name: String, handler: Box<dyn SystemHandler>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.system_channels.insert(system_name.clone(), tx);
        
        // Spawn the system's task with the trait object
        tokio::spawn(async move {
            handler.run(rx).await;  // This just waits for incoming messages
        });
    }
    
    pub fn publish(&self, message: Message) {
        let router = self.message_router.load();
        
        // Find all systems that want this message type
        if let Some(subscribers) = router.get(&message.id) {
            for system_name in subscribers {
                if let Some(sender) = self.system_channels.get(system_name) {
                    let _ = sender.send(message.clone());
                }
            }
        }
    }
}
