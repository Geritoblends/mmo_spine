use tokio::sync::{broadcast, mpsc};

#[derive(ThisError, Debug)]
pub enum Error {

    #[error("Error sending command: {0}")]
    MpscSender(#[from] mpsc::error::SendError),

    #[error("Type mismatch while downcasting: expected: {expected}.")]
    DowncastingError{
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
        } else {
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

