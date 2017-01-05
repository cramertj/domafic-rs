#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Event {
    //TODO
}

pub trait Listener {
    type Message;
    fn event_type_handled(&self) -> &'static str;
    fn handle_event(&self, Event) -> Self::Message;
}

pub struct FnListener<M, F: Fn(Event) -> M> {
    event_type_handled: &'static str,
    f: F,
}

impl<M, F: Fn(Event) -> M> Listener for FnListener<M, F> {
    type Message = M;
    fn event_type_handled(&self) -> &'static str {
        self.event_type_handled
    }
    fn handle_event(&self, event: Event) -> Self::Message {
        (self.f)(event)
    }
}

pub fn on<M, F: Fn(Event) -> M>(event_type: &'static str, f: F) -> FnListener<M, F>
{
    FnListener { event_type_handled: event_type, f: f }
}
