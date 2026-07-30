use crate::tracing::capture::Record;
use crate::tracing::enrichment::{Registry, ResolvedRecord, Resolver};
use crate::tracing::writers::Writer;

use crossbeam::channel::Receiver;

// ── Subscriber ────────────────────────────────────────────────────────────────

pub struct Subscriber<'a> {
    receiver: Receiver<Record>,
    resolver: Resolver<'a>,
    resolved_records: Vec<ResolvedRecord>,

    writers: Vec<Box<dyn Writer>>,
}

impl<'a> Subscriber<'a> {
    pub fn new(receiver: Receiver<Record>, registry: &'a Registry) -> Self {
        Self {
            receiver,
            resolver: Resolver::new(registry),
            resolved_records: Vec::new(),

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
        while let Ok(record) = self.receiver.recv() {
            self.resolver.resolve(&record, &mut self.resolved_records);
        }

        self.dispatch();
    }

    pub fn drain(&mut self) {
        while let Ok(record) = self.receiver.try_recv() {
            self.resolver.resolve(&record, &mut self.resolved_records);
        }

        self.dispatch();
    }

    fn dispatch(&mut self) {
        if !self.writers.is_empty() {
            for writers in &mut self.writers {
                writers.write(&self.resolved_records);
            }
        }
    }
}
