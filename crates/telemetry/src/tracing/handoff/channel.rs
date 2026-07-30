use crate::tracing::capture::Record;

use crossbeam::channel::{bounded, unbounded, Receiver, Sender};

// ── Channels ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Channel {
    pub sender: Sender<Record>,
    pub receiver: Receiver<Record>,
}

impl Channel {
    pub fn bounded(bound: usize) -> Self {
        let (tx, rx) = bounded(bound);

        Self {
            sender: tx,
            receiver: rx,
        }
    }

    pub fn unbounded() -> Self {
        let (tx, rx) = unbounded();

        Self {
            sender: tx,
            receiver: rx,
        }
    }
}
