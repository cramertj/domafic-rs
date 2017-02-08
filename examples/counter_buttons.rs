extern crate domafic;

#[cfg(not(target_os = "emscripten"))]
fn main() {
    panic!("This example needs to be run in the browser via the asm.js or WebAssembly targets.")
}

#[cfg(target_os = "emscripten")]
fn main() {
    use domafic::{DomNode, KeyIter};
    use domafic::tags::{button, div, h1};
    use domafic::listener::on;
    use domafic::web_render::{run, JsIo};

    enum Msg {
        Increment,
        Decrement,
    }

    let update_button = |state: &mut isize, msg: Msg| {
        *state = match msg {
            Msg::Increment => *state + 1,
            Msg::Decrement => *state - 1,
        }
    };

    let render_button = |state: &isize| {
        div ((
            button ((
                on("click", |_| Msg::Decrement),
                "-",
            )),
            state.to_string(),
            button ((
                on("click", |_| Msg::Increment),
                "+",
            )),
        ))
    };

    let update = |state: &mut Vec<isize>, msg: Msg, mut keys: KeyIter, _: &JsIo<Msg>| {
        let key = keys.next().unwrap();
        update_button(&mut state[key], msg)
    };

    let render = |state: &Vec<isize>| {
        div ((
            h1("Hello from rust!"),
            state
                .iter().enumerate()
                .map(|(index, count)| render_button(count).with_key(index))
                .collect::<Vec<_>>()
        ))
    };

    run("body", update, render, vec![0; 10]);
}
