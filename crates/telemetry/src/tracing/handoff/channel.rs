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

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::Channel;
    use crate::tracing::capture::{Record, ScopeId};

    fn record(instance_id: u32) -> Record {
        Record::new(instance_id, None, ScopeId::new(0x1201, 1), 0)
    }

    #[test]
    fn bounded() {
        let channel = Channel::bounded(1);

        let first = record(1);
        channel.sender.send(first.clone()).unwrap();

        assert!(channel.sender.try_send(record(2)).is_err());

        assert_eq!(channel.receiver.recv().unwrap(), first);

        let second = record(2);
        channel.sender.try_send(second.clone()).unwrap();
        assert_eq!(channel.receiver.recv().unwrap(), second);
    }

    #[test]
    fn unbounded() {
        let channel = Channel::unbounded();

        for instance_id in 0..4 {
            channel.sender.send(record(instance_id)).unwrap();
        }

        for instance_id in 0..4 {
            assert_eq!(channel.receiver.recv().unwrap().instance_id, instance_id);
        }

        assert!(channel.receiver.try_recv().is_err());
    }
}
