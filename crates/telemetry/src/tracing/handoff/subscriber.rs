use crate::tracing::capture::Record;
use crate::tracing::enrichment::{Registry, ResolvedRecord, Resolver};
use crate::tracing::writers::Writer;

use crossbeam::channel::Receiver;

// ── Subscriber ────────────────────────────────────────────────────────────────

pub struct Subscriber<'a> {
    receiver: Receiver<Record>,
    resolver: Resolver<'a>,

    writers: Vec<Box<dyn Writer>>,
}

impl<'a> Subscriber<'a> {
    pub fn new(receiver: Receiver<Record>, registry: &'a Registry) -> Self {
        Self {
            receiver,
            resolver: Resolver::new(registry),

            writers: Vec::new(),
        }
    }

    pub fn add_writer<W>(&mut self, writer: W)
    where
        W: Writer,
    {
        self.writers.push(Box::new(writer));
    }
}

impl<'a> Subscriber<'a> {
    pub fn run(&mut self) {
        let mut resolved_records = Vec::new();

        while let Ok(record) = self.receiver.recv() {
            self.resolver.resolve(&record, &mut resolved_records);
        }

        self.dispatch(&resolved_records);
    }

    pub fn drain(&mut self) {
        let mut resolved_records = Vec::new();

        while let Ok(record) = self.receiver.try_recv() {
            self.resolver.resolve(&record, &mut resolved_records);
        }

        self.dispatch(&resolved_records);
    }

    fn dispatch(&mut self, resolved_records: &[ResolvedRecord]) {
        for writer in &mut self.writers {
            writer.write(resolved_records);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::Subscriber;
    use crate::schema::metrics::EVENTS;
    use crate::schema::scopes::INITIALIZATION;
    use crate::tracing::capture::{Metric, Record};
    use crate::tracing::enrichment::{Registry, ResolvedRecord};
    use crate::tracing::writers::Writer;
    use crossbeam::channel;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct SpyWriter {
        batches: Arc<Mutex<Vec<Vec<ResolvedRecord>>>>,
    }

    impl SpyWriter {
        fn batches(&self) -> Vec<Vec<ResolvedRecord>> {
            self.batches.lock().unwrap().clone()
        }
    }

    impl Writer for SpyWriter {
        fn write(&mut self, records: &[ResolvedRecord]) {
            self.batches.lock().unwrap().push(records.to_vec());
        }
    }

    fn record(instance_id: u32) -> Record {
        let mut record = Record::new(instance_id, None, INITIALIZATION, 0);

        record.add_metric(Metric::new(EVENTS, instance_id as u64, 0));

        record
    }

    fn subscriber<'a>(
        receiver: channel::Receiver<Record>,
        registry: &'a Registry,
        writer: &SpyWriter,
    ) -> Subscriber<'a> {
        let mut subscriber = Subscriber::new(receiver, registry);

        subscriber.add_writer(writer.clone());

        subscriber
    }

    #[test]
    fn new() {
        let (_sender, receiver) = channel::unbounded();
        let registry = Registry::default();

        let subscriber = Subscriber::new(receiver, &registry);

        assert!(subscriber.writers.is_empty());
    }

    #[test]
    fn add_writer() {
        let (_sender, receiver) = channel::unbounded();
        let registry = Registry::default();

        let first = SpyWriter::default();
        let second = SpyWriter::default();

        let mut subscriber = Subscriber::new(receiver, &registry);
        subscriber.add_writer(first.clone());
        subscriber.add_writer(second.clone());

        subscriber.drain();

        assert_eq!(first.batches(), vec![Vec::new()]);
        assert_eq!(second.batches(), vec![Vec::new()]);
    }

    #[test]
    fn run() {
        let (sender, receiver) = channel::unbounded();
        let registry = Registry::default();

        let writer = SpyWriter::default();
        let mut subscriber = subscriber(receiver, &registry, &writer);

        sender.send(record(1)).unwrap();
        sender.send(record(2)).unwrap();
        drop(sender);

        subscriber.run();

        let batches = writer.batches();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[0][0].scope_name, "initialization");
        assert_eq!(batches[0][1].scope_name, "initialization");
    }

    #[test]
    fn drain() {
        let (sender, receiver) = channel::unbounded();
        let registry = Registry::default();

        let writer = SpyWriter::default();
        let mut subscriber = subscriber(receiver, &registry, &writer);

        sender.send(record(1)).unwrap();
        sender.send(record(2)).unwrap();

        subscriber.drain();

        let batches = writer.batches();

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[0][0].scope_name, "initialization");
        assert_eq!(batches[0][0].metrics[0].name, "events");
        assert_eq!(batches[0][0].metrics[0].value, 1);
        assert_eq!(batches[0][1].metrics[0].value, 2);

        // Regression: draining again must not re-emit previously resolved records.
        subscriber.drain();

        let batches = writer.batches();

        assert_eq!(batches.len(), 2);
        assert!(batches[1].is_empty());
    }
}
