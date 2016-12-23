pub enum EventType {
    Mouse(MouseEventType),
    Form(FormEventType),
    Focus(FocusEventType),
}

pub enum MouseEventType {
    Click,
    DoubleClick,
    Down,
    Up,
    Enter,
    Leave,
    Over,
    Out,
}

pub enum FormEventType {
    Input,
    Check,
    Submit,
}

pub enum FocusEventType {
    Blur,
    Focus,
}

// TODO
pub struct Event {}
