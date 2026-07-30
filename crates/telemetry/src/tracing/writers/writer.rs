use crate::tracing::enrichment::ResolvedRecord;

pub trait Writer: Send + 'static {
    fn write(&mut self, records: &[ResolvedRecord]);
}
