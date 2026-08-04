use crate::TextRange;

/// Redacted event emitted by an optional diagnostic trace sink.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent {
    pub stage: String,
    pub rule_id: Option<String>,
    pub range: Option<TextRange>,
    pub detail: String,
}

pub trait TraceSink {
    fn record(&mut self, event: TraceEvent);
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VecTrace {
    events: Vec<TraceEvent>,
}

impl VecTrace {
    #[must_use]
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }
}

impl TraceSink for VecTrace {
    fn record(&mut self, event: TraceEvent) {
        self.events.push(event);
    }
}
