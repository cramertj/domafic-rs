use processors::{Listeners, ListenerProcessor};

// TODO make it possible to add fields w/o API breakage
// Consider single private field and unexported `new` fn.
/// Description of a `DOM` event that caused a listener to be called.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Event<'a> {
    /// Type of event
    pub type_str: Option<&'a str>,
    /// Value of the node from which the event originated
    pub target_value: Option<&'a str>,
    /// Horizontal component at which the event occurred relative to the client area
    pub client_x: i32,
    /// Vertical component at which the event occurred relative to the client area
    pub client_y: i32,
    /// Horizontal component at which the event occurred relative to the target node
    pub offset_x: i32,
    /// Vertical component at which the event occurred relative to the target node
    pub offset_y: i32,
    /// Keycode of the keyboard key or mouse button that caused the event
    pub which_keycode: i32,
    /// Whether or not the "shift" key was pressed at the time of the event
    pub shift_key: bool,
    /// Whether or not the "alt" key was pressed at the time of the event
    pub alt_key: bool,
    /// Whether or not the "ctrl" key was pressed at the time of the event
    pub ctrl_key: bool,
    /// Whether or not the "meta" key was pressed at the time of the event
    pub meta_key: bool,
}

/// `Listener`s listen to events and convert them into a message
pub trait Listener<Message> {
    /// Type of event handled by this `Listener`. Example: "click".
    fn event_type_handled(&self) -> &'static str;
    /// Handle a given event, producing a message
    fn handle_event(&self, Event) -> Message;
}

/// A listener that consists of an event type and a function from `Event` to message
pub struct FnListener<M, F: Fn(Event) -> M> {
    event_type_handled: &'static str,
    f: F,
}

impl<M, F: Fn(Event) -> M> Listeners<M> for FnListener<M, F> {
    fn process_all<'a, P: ListenerProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}

impl<M, F: Fn(Event) -> M> Listener<M> for FnListener<M, F> {
    fn event_type_handled(&self) -> &'static str {
        self.event_type_handled
    }
    fn handle_event(&self, event: Event) -> M {
        (self.f)(event)
    }
}

/// Create an `FnListener` that handles to events of type `event_type` using function `f`
pub fn on<M, F: Fn(Event) -> M>(event_type: &'static str, f: F) -> FnListener<M, F>
{
    FnListener { event_type_handled: event_type, f: f }
}
