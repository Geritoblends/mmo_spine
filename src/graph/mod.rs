use tokio::sync::{broadcast, mpsc};

pub struct Handler<I, O, D> {
    id: &'static str
    input: mpsc::Receiver<I>,
    output: broadast::Sender<O>,
    channels: Arc<HashMap<&'static str, mpsc::Sender<D>>>, // D for Dependency Enum (maps to deps)
}

