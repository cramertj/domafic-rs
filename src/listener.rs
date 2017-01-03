use events;

pub trait Listener {
    type Message;
    fn event_types_handled(&self) -> &[events::EventType];
    fn handle_event(&self, events::Event) -> Self::Message;
}

pub struct FnListener<M, A: AsRef<[events::EventType]>, F: Fn(events::Event) -> M> {
    events_handled: A,
    f: F,
}

impl<
    M,
    A: AsRef<[events::EventType]>,
    F: Fn(events::Event) -> M> Listener for FnListener<M, A, F>
{
    type Message = M;
    fn event_types_handled(&self) -> &[events::EventType] {
        self.events_handled.as_ref()
    }
    fn handle_event(&self, event: events::Event) -> Self::Message {
        (self.f)(event)
    }
}

pub fn on<M, F: Fn(events::Event) -> M>
    (event_type: events::EventType, f: F) -> FnListener<M, [events::EventType; 1], F>
{
    FnListener { events_handled: [event_type], f: f }
}

pub fn on_events<M, A: AsRef<[events::EventType]>, F: Fn(events::Event) -> M>
    (events_handled: A, f: F) -> FnListener<M, A, F>
{
    FnListener { events_handled: events_handled, f: f }
}
