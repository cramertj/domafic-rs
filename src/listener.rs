// TODO make it possible to add fields w/o API breakage
// Consider single private field and unexported `new` fn.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Event<'a> {
    pub type_str: Option<&'a str>,
    pub target_value: Option<&'a str>,
    pub client_x: i32,
    pub client_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub which_keycode: i32,
    pub shift_key: bool,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
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
