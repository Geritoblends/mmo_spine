#[derive(Serialize, Deserialize)]
struct Message {
    #[serde(borrow)]
    id: &'a str,
    #[serde(borrow)]
    bincode_payload: &'a [u8],
}

impl Message {
    fn get_id(&self) -> &'a str
}

pub trait MessageHandler {
    fn get_id(&self) -> String;
    fn handle(&self, msg: Message);
}

pub trait System {
}

pub struct MessageBus {
    handlers: Arc<Mutex<HashMap<String, Vec<Box<dyn MessageHandler>>>>>, // Allocation is okay because they're handlers
                                                    // that persist through the whole program,
                                                    // compared to Messages that are possibly
                                                    // real-time
}

impl MessageBus {

    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn subscribe(&self, handler: Box<dyn MessageHandler>) {
        let handlers = self.handlers.lock();
        let wrapped_handler = Box::new(handler);
        handlers
            .entry(handler.get_id())
            .or_insert_with(Vec::new)
            .push(wrapped_handler);
    }

    pub fn publish(&self, msg: Message) {
        let handlers = self.handlers.lock();

        if let Some(handler_list) = handlers.get(&msg.get_id()) {
            for handler in handler_list {
                handler.handle(msg)
            }
        }
    }

}
        
