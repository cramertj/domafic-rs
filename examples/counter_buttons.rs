extern crate domafic;
use domafic::{DOMNode, IntoNode};
use domafic::tags::{div, button};
use domafic::events::EventType::Click;
use domafic::listener::on;

type State = usize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
    let update = |state: State, msg: Msg| match msg {
        Msg::Increment => state + 1,
        Msg::Decrement => state - 1,
    };

    let render = |state: State| {
        div ((
            button ((
                on(Click, |_| Msg::Decrement),
                "-".into_node(),
            )),
            state.to_string().into_node(),
            button ((
                on(Click, |_| Msg::Increment),
                "+".into_node(),
            )),
        ))
    };
}
