#![feature(conservative_impl_trait)]
extern crate domafic;
use domafic::{DOMNode, IntoNode};
use domafic::tags::{div, button};

type State = usize;

enum Msg {
    Increment,
    Decrement,
}

fn update(state: State, msg: Msg) -> State {
    match msg {
        Msg::Increment => state + 1,
        Msg::Decrement => state - 1,
    }
}

fn render(state: State) -> impl DOMNode<Message=Msg> {
    div ((
        button (("-".into_node())),
        button (("+".into_node())),
    ))
}

fn main() {
}
