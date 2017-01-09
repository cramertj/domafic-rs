//! # Domafic - Safe, high-performance, universal web applications
//!
//! Domafic is a library for building templates and interactive web applications. Applications
//! built in Domafic can be rendered server-side and used in front-end web servers, but they can
//! also be deployed directly to the client using asm.js and WebAssembly.
//!
//! A simple template:
//!
//! ```rust
//! use domafic::{DOMNode, IntoNode};
//! use domafic::tags::{div, h1};
//! use domafic::empty::empty;
//!
//! type Msg = ();
//!
//! // Create a function `render` from `birthday: &'static str` to `DOMNode<Message=Msg>`
//! let render = |birthday: &'static str| div((
//!     h1((
//!         "Hello, world! Your birthday is: ".into_node(),
//!         birthday.into_node(),
//!     )),
//!
//!     // Since we don't publish any messages, we need to create an empty node with our
//!     // message type. This that the compiler that our message type is `Msg`. This would
//!     // be unnecessary if we publshed any messages or if we specified the return type of
//!     // the `render` function.
//!     empty::<Msg>(),
//! ));
//!
//! assert_eq!(
//!     "<div><h1>Hello, world! Your birthday is: Christmas</h1></div>".to_string(),
//!     render("Christmas").to_string()
//! );
//! ```
//!
//! If you've used HTML or JSX, the syntax should look familiar. Note that we didn't need
//! to use any macros or interpreters-- the template above is just pure, allocation-free Rust. The
//! template itself is just a function that returns a `DOMNode`. The `DOMNode` trait lets us use
//! the result of `render` as an HTML node. We can write `DOMNode`s to HTML, render them to a live
//! web page using Javascript, or use them as children of other `DOMNode`s.
//!
//! Domafic's design is similar to that of popular single-state frontend frameworks such as Elm
//! or Redux. An application consists of state, an updater, and a renderer.
//!
//! The application state holds all of the information needed by the renderer to draw the page.
//! The renderer is a function that takes the current state as input and produces the current UI as
//! output. Finally, the updater is responsible for recieving messages generated by event listeners
//! and updating the application state accordingly.
//!
//! For example, here is a simple example showing a counter and +/- buttons:
//!
//! ```rust
//! use domafic::IntoNode;
//! use domafic::tags::{button, div, h1};
//! use domafic::listener::on;
//!
//! // If rendering client-side with asm.js or WebAssembly:
//! #[cfg(target_os = "emscripten")]
//! use domafic::web_render::run;
//! #[cfg(target_os = "emscripten")]
//! use domafic::KeyIter;
//!
//! type State = isize;
//!
//! enum Msg {
//!     Increment,
//!     Decrement,
//! }
//!
//! #[cfg(target_os = "emscripten")]
//! let update = |state: &mut State, msg: Msg, _keys: KeyIter| {
//!     *state = match msg {
//!         Msg::Increment => *state + 1,
//!         Msg::Decrement => *state - 1,
//!     }
//! };
//!
//! let render = |state: &State| {
//!     div ((
//!         h1("Hello from rust!".into_node()),
//!         button ((
//!             on("click", |_| Msg::Decrement),
//!             "-".into_node(),
//!         )),
//!         state.to_string().into_node(),
//!         button ((
//!             on("click", |_| Msg::Increment),
//!             "+".into_node(),
//!         )),
//!     ))
//! };
//!
//! // If rendering server-side:
//! #[cfg(not(target_os = "emscripten"))]
//! println!("HTML: {}", render(&0));
//!
//! // If rendering client-side with asm.js or WebAssembly:
//! #[cfg(target_os = "emscripten")]
//! run("body", update, render, 0);
//! ```
//!
//! Check out more examples like this one
//! [in the Github repository.](https://github.com/cramertj/domafic-rs/tree/master/examples)
//!
//! The above example, if compiled for an emscripten target
//! (via `cargo build --target=asmjs-unknown-emscripten` or similar) will produce a Javascript file
//! that, when included on a webpage, will replace the contents of "body" with the message
//! "Hello from rust!", +/- buttons, and a number.
//!
//! So how does this all work? When the call to `run` occurs, Domafic gives the initial state (0)
//! to the renderer (our "render" function) which returns the initial page to display to the user.
//!
//! This page includes buttons with listeners for `on("click", ...)`, so when a button is clicked,
//! the appropriate message is generated (either `Msg::Increment` or `Msg::Decrement`). This
//! message is then passed into the updater (our `update` function) and used to update the state.
//!
//! Once the state is successfully updated, `render` is called once more to redraw the page.
//! When run in the browser, Domafic keeps an internal DOM (tree-based representation of the UI)
//! and uses it to minimize the changes that need to be made on-screen. This prevents unnecessary
//! re-drawing of UI components.
//!
//! One last thing you may have noticed:
//! we've been writing our `render` functions as closures, rather than named functions.
//! The reason for this is that the return type of the `render` method is long and hard
//! to write out. If you must use named functions, consider using the nightly
//! `conservative_impl_trait` feature, which will allow you to write the function signature of
//! `render` like `fn render(state: &State) -> impl DOMNode<Message=Msg>`.

#![cfg_attr(test, feature(conservative_impl_trait))]
#![cfg_attr(not(any(feature = "use_std", test)), no_std)]
#![allow(unused_unsafe)]
#![deny(missing_docs)]

/// Trait for elements that can be drawn as to HTML DOM nodes
pub mod dom_node;
pub use dom_node::{DOMNode, DOMValue, IntoNode};

#[cfg(any(feature = "use_std", test))]
/// Types, traits and functions for writing a `DOMNode` to HTML
pub mod html_writer;

mod keys;
pub use keys::KeyIter;
/// Types, traits, and functions for creating event handlers
pub mod listener;
pub use listener::{Listener, Event, on};
/// Traits for processing collections of `DOMNode`s or `Listener`s
pub mod processors;
pub use processors::{DOMNodes, Listeners};
/// Types and functions for creating tag elements such as `div`s or `span`s
pub mod tags;

#[cfg(feature = "web_render")]
/// Functions for interacting with a webpage when rendering client-side using asmjs or emscripten
pub mod web_render;

/// A mapping between an attribute key and value.
/// Example: `("key", AttributeValue::Str("value"))`
pub type KeyValue = (&'static str, AttributeValue);

/// A value of a `DOMNode` attribute.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AttributeValue {
    /// A value represented by a static string reference
    Str(&'static str),
    /// A value represented by an owned `String`
    OwnedStr(String),
    /// A boolean value
    Bool(bool),

    // TODO: add numeric variants?
}

impl AttributeValue {
    /// Extracts a string slice representing the contents.
    /// If the value is a `Bool`, this method returns "true" or "false".
    fn as_str(&self) -> &str {
        match *self {
            AttributeValue::Str(value) => value,
            AttributeValue::OwnedStr(ref value) => value,
            AttributeValue::Bool(true) => "true",
            AttributeValue::Bool(false) => "false",
        }
    }
}

#[cfg(any(feature = "use_std", test))]
impl std::fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Types and functions for creating `DOMNodes` or `Listeners` with no runtime representation.
pub mod empty {
    #[cfg(not(any(feature = "use_std", test)))]
    extern crate core as std;
    use std::marker::PhantomData;

    use super::processors::{DOMNodes, DOMNodeProcessor, Listeners, ListenerProcessor};

    /// An empty set of nodes with no children or attributes.
    /// Instances of this type have no DOM representation.
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub struct EmptyNodes<Message>(pub PhantomData<Message>);

    /// Creates a new `EmptyNodes`.
    pub fn empty<Message>() -> EmptyNodes<Message> { EmptyNodes(PhantomData) }

    impl<M> DOMNodes for EmptyNodes<M> {
        type Message = M;
        fn process_all<'a, P: DOMNodeProcessor<'a, M>>(&'a self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
    }

    /// An empty set of listeners.
    /// Instances of this type have no DOM representation.
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
    pub struct EmptyListeners<Message>(pub PhantomData<Message>);

    /// Creates a new `EmptyListeners`.
    pub fn empty_listeners<Message>() -> EmptyListeners<Message> { EmptyListeners(PhantomData) }
    impl<M> Listeners for EmptyListeners<M> {
        type Message = M;
        fn process_all<'a, P: ListenerProcessor<'a, Self::Message>>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DOMNode, DOMValue, KeyValue, IntoNode};
    use super::AttributeValue::Str;
    use super::tags::*;
    use super::processors::{DOMNodes, DOMNodeProcessor};
    use super::empty::{empty, empty_listeners, EmptyNodes, EmptyListeners};
    use super::html_writer::HtmlWriter;

    #[cfg(feature = "use_either_n")]
    extern crate either_n;
    #[cfg(feature = "use_either_n")]
    use self::either_n::*;

    use std::marker::PhantomData;

    struct BogusOne(EmptyNodes<Never>, EmptyListeners<Never>);
    const BOGUS_1: BogusOne = BogusOne(EmptyNodes(PhantomData), EmptyListeners(PhantomData));
    impl DOMNode for BogusOne {
        type Message = Never;
        type Children = EmptyNodes<Self::Message>;
        type Listeners = EmptyListeners<Self::Message>;
        type WithoutListeners = BogusOne;

        fn children(&self) -> &Self::Children { &self.0 }
        fn listeners(&self) -> &Self::Listeners { &self.1 }
        fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
            (&self.0, &self.1)
        }
        fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
            (BOGUS_1, empty_listeners())
        }

        fn key(&self) -> Option<u32> { None }
        fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
        fn value(&self) -> DOMValue {
            DOMValue::Element { tag: "bogus_tag_one" }
        }
    }

    struct BogusTwo(EmptyNodes<Never>, EmptyListeners<Never>);
    const BOGUS_2: BogusTwo = BogusTwo(EmptyNodes(PhantomData), EmptyListeners(PhantomData));
    impl DOMNode for BogusTwo {
        type Message = Never;
        type Children = EmptyNodes<Self::Message>;
        type Listeners = EmptyListeners<Self::Message>;
        type WithoutListeners = BogusTwo;

        fn key(&self) -> Option<u32> { None }
        fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }

        fn children(&self) -> &Self::Children { &self.0 }
        fn listeners(&self) -> &Self::Listeners { &self.1 }
        fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
            (&self.0, &self.1)
        }
        fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
            (BOGUS_2, empty_listeners())
        }

        fn value(&self) -> DOMValue {
            DOMValue::Element { tag: "bogus_tag_two" }
        }
    }

    struct ChildCounter;
    #[derive(Debug, Clone, Copy)]
    enum Never {}
    impl<'a, M> DOMNodeProcessor<'a, M> for ChildCounter {
        type Acc = usize;
        type Error = Never;

        fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &'a T) -> Result<(), Never> {
            fn incr<'a, T: DOMNode>(count: &mut usize, _node: &'a T) -> Result<(), Never> {
                *count += 1;
                Ok(())
            }
            incr
        }
    }

    fn html_sample() -> impl DOMNode<Message = Never> {
        div ((
            attributes([("attr", Str("value"))]),
            (
            BOGUS_1,
            BOGUS_1,
            BOGUS_2,
            table ((
                "something".into_node(),
                th (empty()),
                tr (empty()),
                tr (empty()),
            )),
            )
        ))
    }

    #[cfg(feature = "use_either_n")]
    fn html_either(include_rows: bool) -> impl DOMNode<Message = Never> {
        div((
            table((
                if include_rows {
                    Either2::One((
                        tr("a".into_node()),
                        tr("b".into_node()),
                    ))
                } else {
                    Either2::Two("sumthin else".into_node())
                }
            ))
        ))
    }

    #[cfg(feature = "use_either_n")]
    fn builds_an_either_string(arg: bool, expected: &'static str) {
        let mut string_buffer = Vec::new();
        html_either(arg).process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer).unwrap();
        let string = String::from_utf8(string_buffer).unwrap();
        assert_eq!(
            without_whitespace(expected.to_string()),
            without_whitespace(string)
        );
    }

    #[cfg(feature = "use_either_n")]
    #[test]
    fn builds_either_string() {
        builds_an_either_string(true, r#"
        <div>
            <table>
                <tr>a</tr>
                <tr>b</tr>
            </table>
        </div>
        "#);

        builds_an_either_string(false, r#"
        <div>
            <table>
                sumthin else
            </table>
        </div>
        "#);
    }

    #[test]
    fn counts_children() {
        let mut count = 0;
        (BOGUS_1, BOGUS_1, BOGUS_2).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        (BOGUS_1, (BOGUS_1,), BOGUS_2).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        [BOGUS_1, BOGUS_1, BOGUS_1].process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        (BOGUS_1, BOGUS_1,
            [BOGUS_1, BOGUS_1, BOGUS_1],
            [(BOGUS_1)],
            vec![empty(), empty(), empty()],
            [BOGUS_2, BOGUS_2, BOGUS_2],
        ).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(9, count);

        let sample = html_sample();

        count = 0;
        sample.process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(1, count);

        count = 0;
        sample.children().process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(4, count);
    }

    fn without_whitespace(string: String) -> String {
        string.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn builds_string() {
        let mut string_buffer = Vec::new();
        html_sample().process_all::<HtmlWriter<Vec<u8>>>(&mut string_buffer).unwrap();
        let string = String::from_utf8(string_buffer).unwrap();
        assert_eq!(
            without_whitespace(r#"
            <div attr="value">
                <bogus_tag_one></bogus_tag_one>
                <bogus_tag_one></bogus_tag_one>
                <bogus_tag_two></bogus_tag_two>
                <table>
                    something
                    <th></th>
                    <tr></tr>
                    <tr></tr>
                </table>
            </div>
            "#.to_string()),
            without_whitespace(string)
        );
    }

    fn check_attribute_list<T: DOMNode>(div: T) {
        assert_eq!(div.get_attribute(0), Some(&("attr1", Str("val1"))));
        assert_eq!(div.get_attribute(1), Some(&("attr2", Str("val2"))));
        assert_eq!(div.get_attribute(2), Some(&("attr3", Str("val3"))));
        assert_eq!(div.get_attribute(3), None);

        let mut attr_iter = div.attributes();
        assert_eq!(attr_iter.next(), Some(&("attr1", Str("val1"))));
        assert_eq!(attr_iter.next(), Some(&("attr2", Str("val2"))));
        assert_eq!(attr_iter.next(), Some(&("attr3", Str("val3"))));
        assert_eq!(attr_iter.next(), None);
    }

    #[test]
    fn builds_attribute_list() {
        let div1 = div(empty::<Never>())
            .with_attributes([("attr2", Str("val2")), ("attr3", Str("val3"))])
            .with_attributes([("attr1", Str("val1"))]);
        check_attribute_list(div1);

        let div2 = div((
            attributes([("attr2", Str("val2")), ("attr3", Str("val3"))]),
            div(empty::<Never>())
        )).with_attributes([("attr1", Str("val1"))]);
        check_attribute_list(div2);
    }
}
