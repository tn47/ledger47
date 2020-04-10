use chrono;
use llrb_index::Llrb;

use ledger::{
    core::{Error, Result},
    err_at,
};

use std::sync::mpsc;

#[derive(Clone)]
pub enum Event {
    Date(chrono::Date<chrono::Local>),
    Period(chrono::Date<chrono::Local>, chrono::Date<chrono::Local>),
    StatusError(String),
}

pub enum Tx {
    N(mpsc::Sender<Event>),
    S(mpsc::SyncSender<Event>),
}

impl Tx {
    pub fn new() -> (Tx, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel();
        (Tx::N(tx), rx)
    }

    pub fn new_sync(channel_size: usize) -> (Tx, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::sync_channel(channel_size);
        (Tx::S(tx), rx)
    }
}

impl Clone for Tx {
    fn clone(&self) -> Self {
        match self {
            Tx::N(tx) => Tx::N(tx.clone()),
            Tx::S(tx) => Tx::S(tx.clone()),
        }
    }
}

pub struct PubSub {
    subscribers: Llrb<String, Vec<Tx>>,
}

impl PubSub {
    pub fn new(name: &str) -> PubSub {
        PubSub {
            subscribers: Llrb::new(name),
        }
    }

    pub fn subscribe(&mut self, topic: &str, tx: Tx) {
        let value = match self.subscribers.get(topic) {
            Some(mut value) => {
                value.push(tx);
                value
            }
            None => vec![tx],
        };
        self.subscribers.set(topic.to_string(), value);
    }

    pub fn publish(&mut self, topic: &str, event: Event) -> Result<()> {
        match self.subscribers.get(topic) {
            Some(txs) => {
                for tx in txs {
                    match tx {
                        Tx::N(tx) => err_at!(IOError, tx.send(event.clone()), format!("publish"))?,
                        Tx::S(tx) => err_at!(IOError, tx.send(event.clone()), format!("publish"))?,
                    }
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}
