use std::sync::mpsc::{Sender, Receiver, SendError, TryRecvError};
use std::sync::mpsc::channel as single_channel;

pub struct Endpoint<TOut, TIn> {
    link_out: Sender<TOut>,
    link_in:  Receiver<TIn>,
}

impl<TOut, TIn> Endpoint<TOut, TIn> {
    fn new(sender: Sender<TOut>, receiver: Receiver<TIn>) ->  Endpoint<TOut, TIn> {
        Endpoint {
            link_out: sender,
            link_in:  receiver,
        }
    }

    pub fn send(&self, message: TOut) -> Result<(), SendError<TOut>> {
        self.link_out.send(message)
    }

    pub fn try_recv(&self) -> Result<TIn, TryRecvError> {
        self.link_in.try_recv()
    }
}

pub fn channel<T1, T2>() -> (Endpoint<T1, T2>, Endpoint<T2, T1>) {
    let (sender_t1, receiver_t1) = single_channel::<T1>();
    let (sender_t2, receiver_t2) = single_channel::<T2>();
    (
        Endpoint::new(sender_t1, receiver_t2),
        Endpoint::new(sender_t2, receiver_t1)
    )
}
