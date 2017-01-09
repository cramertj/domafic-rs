extern crate domafic;

#[cfg(not(target_os = "emscripten"))]
fn main() {
    panic!("This example needs to be run in the browser via the asm.js or WebAssembly targets.")
}

#[cfg(target_os = "emscripten")]
fn main() {
    use domafic::{DomNode, KeyIter, IntoNode};
    use domafic::AttributeValue::*;
    use domafic::tags::*;
    use domafic::listener::on;
    use domafic::web_render::run;

    enum Msg {
        UpdateField(String),
        Add(String),
        Remove,
        None,
    }

    struct TodoState {
        entry_box: String,
        todos: Vec<String>,
    }
    impl TodoState {
        fn new() -> TodoState {
            TodoState {
                entry_box: String::new(),
                todos: Vec::new(),
            }
        }
    }

    let update = |state: &mut TodoState, msg: Msg, mut keys: KeyIter| {
        match msg {
            Msg::UpdateField(value) => {
                state.entry_box = value
            },
            Msg::Add(todo) => {
                state.entry_box = String::new();
                state.todos.push(todo);
            },
            Msg::Remove => {
                state.todos.remove(keys.next().unwrap());
            },
            Msg::None => {},
        }
    };

    const ENTER_KEYCODE: i32 = 13;
    let render_todo_input_field = |current_value: &str| {
        input((
            attributes([
                ("type", Str("text")),
                ("placeholder", Str("What do you have to do?")),
                ("autofocus", Bool(true)),
                ("value", OwnedStr(current_value.to_owned())),
            ]),
            (
                on("input", |event|
                    if let Some(target_value) = event.target_value {
                        Msg::UpdateField(target_value.to_owned())
                    } else { Msg::None }
                ),
                on("keydown", |event|
                    if let (ENTER_KEYCODE, Some(target_value)) =
                        (event.which_keycode, event.target_value)
                    {
                        Msg::Add(target_value.to_owned())
                    } else { Msg::None }
                )
            )
        ))
    };

    let render_item = |state: &str| {
        div ((
            state.to_owned().into_node(),
            button ((
                on("click", |_| Msg::Remove),
                "Remove".into_node(),
            )),
        ))
    };

    let render = |state: &TodoState| {
        div ((
            h1("TODO:".into_node()),
            render_todo_input_field(&state.entry_box),
            state.todos
                .iter().enumerate()
                .map(|(index, todo)| render_item(todo).with_key(index))
                .collect::<Vec<_>>()
        ))
    };

    run("body", update, render, TodoState::new());
}
