// TEMPORARY Test with asmjs
// TODO Replace with examples in /examples once Windows builds are fixed
// so that the manual `cargo rustc ... --linker="emcc.bat"` workaround is
// unnecessary

extern crate domafic;
use domafic::{DOMNode, DOMValue, IntoNode};
use domafic::tags::{button, div, h1};
use domafic::events::EventType::Click;
use domafic::listener::on;

#[macro_use] extern crate webplatform;
extern crate libc;

type State = usize;

enum Msg {
    Increment,
    Decrement,
}

fn main() {
    let update = |state: &mut State, msg: Msg| match msg {
        Msg::Increment => *state += 1,
        Msg::Decrement => *state -= 1,
    };

    let render = |state: &State| {
        div ((
            h1("Hello from rust!".into_node()),
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

    run("body", update, render, 0);
}

trait Updater<State, Message> {
    fn update(&self, &mut State, Message) -> ();
}
impl<F, S, M> Updater<S, M> for F where F: Fn(&mut S, M) -> () {
    fn update(&self, state: &mut S, msg: M) -> () {
        (self)(state, msg)
    }
}

trait Renderer<State> {
    type Rendered: DOMNode;
    fn render(&self, &State) -> Self::Rendered;
}
impl<F, S, R> Renderer<S> for F where F: Fn(&S) -> R, R: DOMNode {
    type Rendered = R;
    fn render(&self, state: &S) -> Self::Rendered {
        (self)(state)
    }
}

fn run<U, R, S>(element_selector: &str, updater: U, renderer: R, initial_state: S) -> !
    where
    U: Updater<S, <<R as Renderer<S>>::Rendered as DOMNode>::Message>,
    R: Renderer<S>
{

    unsafe {
        // Lives forever on the stack, referenced and mutated in callbacks
        let mut app_system = (updater, renderer, initial_state);
        let app_system_mut_ref = &mut app_system;
        let app_system_mut_ptr = app_system_mut_ref as *mut (U, R, S);

        // Borrow only used to draw initial DOM to browser
        let new_app_system_mut_ref = &mut (*app_system_mut_ptr);

        // Get initial DOMNode
        let rendered = new_app_system_mut_ref.1.render(&(new_app_system_mut_ref).2);

        // Initialize the browser system
        let document = webplatform::init();
        let body = document.element_query(element_selector).unwrap();

        // Draw initial DOMNode to browser
        rendered.process_all::<WebPlatformWriter<U, R, S>>(
            &mut (app_system_mut_ptr, &document, &body)
        ).unwrap();

        webplatform::spin();
    }

    panic!("webplatform::spin() returned");
}

use std::marker::PhantomData;
use domafic::listener::Listener;
use domafic::processors::{DOMNodes, DOMNodeProcessor, Listeners, ListenerProcessor};
use webplatform::{Document as WebDoc, HtmlNode as WebNode};

type MessageOfR<R: Renderer<S>, S> = <<R as Renderer<S>>::Rendered as DOMNode>::Message;

struct WebPlatformWriter<'a, 'd, U:'d, R:'d, S:'d>(
    PhantomData<(&'a (), &'d (), U, R, S)>
);
impl<'a, 'd: 'a, U:'d, R:'d, S:'d> DOMNodeProcessor<'d, MessageOfR<R, S>> for WebPlatformWriter<'a, 'd, U, R, S>
    where
    U: Updater<S, MessageOfR<R, S>>,
    R: Renderer<S>
{
    type Acc = (*mut (U, R, S), &'a WebDoc<'d>, &'a WebNode<'d>);
    type Error = ();

    fn get_processor<T: DOMNode<Message=MessageOfR<R, S>>>() -> fn(&mut Self::Acc, &'d T) -> Result<(), Self::Error> {

        // Private to this function because it's actually unsafe to use, but there's
        // currently no way to make it unsafe to use a safe trait, and we need to use
        // the ListenerProcessor trait
        struct WebPlatformListenerWriter<
            'a, 'd: 'a,
            U: Updater<S, MessageOfR<R, S>> + 'd,
            R: Renderer<S> + 'd,
            S:'d>
        (
            PhantomData<(&'a (), &'d (), U, R, S)>
        );
        impl<'a, 'd: 'a, U:'d, R:'d, S:'d> ListenerProcessor<'d, MessageOfR<R, S>> for
            WebPlatformListenerWriter<'a, 'd, U, R, S>
            where
            U: Updater<S, MessageOfR<R, S>>,
            R: Renderer<S>
        {
            type Acc = (*mut (U, R, S), &'a WebNode<'d>);
            type Error = ();

            fn get_processor<L: Listener<Message=MessageOfR<R, S>>>() -> fn(&mut Self::Acc, &'d L) -> Result<(), Self::Error> {
                fn add_listener<'a, 'd, U:'d, R:'d, S:'d, L> (
                    acc: &mut (*mut (U, R, S), &'a WebNode<'d>),
                    listener: &'d L) -> Result<(), ()> where L: Listener
                {
                    let (ref boxed_system_ptr_ref, ref node) = *acc;
                    let boxed_system_ptr: *mut (U, R, S) =
                        (*boxed_system_ptr_ref).clone();
                    let listener_ptr = listener as *const L;

                    node.on("click", move |_target| {
                        let boxed_system_mut_ref: &mut (U, R, S) = unsafe {
                            boxed_system_ptr.as_mut().unwrap()
                        };
                        let listener_ref: &L = unsafe {
                            // The listener lives in the boxed_system, so is safe to Deref here
                            // so long as the listener lists in the boxed system aren't mutated
                            // (which they aren't between listener registration and callback)
                            listener_ptr.as_ref().unwrap()
                        };
                        let message = listener_ref.handle_event(domafic::events::Event {});
                        // TODO update
                    });

                    Ok(())
                }
                add_listener
            }
        }

        fn add_node<'a, 'd:'a, T, U:'d, R:'d, S:'d>(
                acc: &mut (*mut (U, R, S), &'a WebDoc<'d>, &'a WebNode<'d>),
                node: &'d T) -> Result<(), ()>
                where
                T: DOMNode<Message=MessageOfR<R, S>>,
                U: Updater<S, MessageOfR<R, S>>,
                R: Renderer<S>
        {
            let (ref boxed_system, ref document, ref parent_node) = *acc;

            match node.value() {
                DOMValue::Element { tag: tagname } => {
                    let html_node = document.element_create(tagname).unwrap();
                    for attr in node.attributes() {
                        html_node.prop_set_str(attr.0, attr.1);
                    }

                    // Reborrow of *document needed to match lifetimes for 'a
                    let new_acc = &mut (*boxed_system, &**document, &html_node);
                    node.children().process_all::<WebPlatformWriter<U, R, S>>(new_acc)?;
                    let (_, _, html_node) = *new_acc;
                    node.listeners().process_all::<WebPlatformListenerWriter<U, R, S>>(
                        &mut (*boxed_system, html_node)
                    )?;
                    parent_node.append(&html_node);
                }
                DOMValue::Text(text) => {
                    // TODO replace with createTextNode (need to add to webplatform API)
                    parent_node.html_append(text);
                }
            }
            Ok(())
        }
        add_node
    }
}
