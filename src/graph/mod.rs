use tokio::sync::{broadcast, mpsc};

pub trait Message {
    fn get_id(&self) -> &'static str;
}

pub enum MessagePayload {
    Small([u8; 64]),
    Large([u8; 1024]),
    Unsized(Box<[u8]>)
}

impl AsRef<[u8]> for MessagePayload {
    fn as_ref(&self) -> &[u8] {
        match self {
            MessagePayload::Small(arr) => arr.as_ref(),
            MessagePayload::Large(arr) => arr.as_ref(), 
            MessagePayload::Unsized(boxed) => boxed.as_ref(),
        }
    }
}

pub struct Message {
    id: &'static str,
    payload: MessagePayload,
    payload_len: usize,
}

impl Message {
    pub fn payload_bytes(&self) -> &[u8] {
        &self.payload.as_ref()[..self.payload_len]

    }

    pub fn parse<T>(&self) -> Result<T, bincode::error::Error>
        where
            T: for<'de> Deserialize<'de>
    {
        let bytes = self.payload_bytes();

        let msg = bincode::deserialize::<T>(&bytes)?;
        Ok(msg)
    }

}


#[derive(ThisError, Debug)]
pub enum Error {

    #[error("Error sending command: {0}")]
    MpscSender(#[from] mpsc::error::SendError),

    #[error("Type mismatch while downcasting: expected: {expected}.")]
    DowncastingError {
        expected: &'static str
    },

}

trait AnyMpscSender {
    fn send_any(&self, msg: &dyn Any) -> Result<(), Error>;
}

trait AnyBroadcastReceiver {
    fn recv_any(&mut self); // Handlers will implement it based on their own side-effect message
                            // dependencies
}

impl<T: Send + Sync + Clone + 'static> AnyMpscSender for mpsc::Sender<T> {
    fn send_any(&self, msg: &dyn Any) -> Result<(), Error> {
        if let Some(typed_msg) = msg.downcast_ref::<T>() {
            self.send(typed_msg.clone())?;
         else {
            return Err(Error::DowncastingError { expected:  std::any::type_name::<T>()) });
        }
        Ok(())
    }
}

pub struct Handler<I, O> {
    id: &'static str,
    command_receiver: mpsc::Receiver<I>,
    command_senders: Arc<HashMap<&'static str, Box<dyn AnyMpscSender>>>,
    message_receivers: Vec<Box<dyn AnyBroadcastReceiver>>,
    message_sender: broadcast::Sender<O>
}

