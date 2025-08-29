struct PlayerDamaged {
    player_id: i32,
    damage: i32,
}

struct MessageBus {
    handlers: HashMap<TypeId, Fn(PlayerDamaged)>
}

impl MessageBus {
    fn new(&self) -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    fn subscribe(&self, handler: &dyn MessageHandler<T>) {
        let type_id = TypeId::from::<T>;
        let handlers = self.handlers.lock();

        handlers
            .entry(&type_id)
            .or_update_with(Vec::new())
            .push(handler);
    }

    fn publish(&self, msg: &dyn Message<T>) {
        let type_id = TypeId::msg.as_any().downcast_ref::<T>

