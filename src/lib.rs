#![cfg_attr(test, feature(conservative_impl_trait))]
#![cfg_attr(not(any(feature = "use_std", test)), no_std)]

#[cfg(not(any(feature = "use_std", test)))]
extern crate core as std;

use std::marker::PhantomData;

/// A `DOMNode` specifies the HTML DOM (Document Object Model) representation of a type.
///
/// Note that there can be many different types that map to the same HTML. For example, both
/// `String` and `str` can be used to create HTML text nodes.
pub trait DOMNode: Sized {

    /// Type of message sent by a listener. Messages of this type should be used to update
    /// application state.
    type Message;

    /// Get the nth attribute for a given `DOMNode`.
    ///
    /// If `node.get_attribute(i)` returns `None`, `node.get_attribute(j)` should return `None`
    /// for all `j >= i`.
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue>;

    /// Returns an iterator over a `DOMNode`'s attributes.
    fn attributes<'a>(&'a self) -> AttributeIter<'a, Self> {
        AttributeIter { node: self, index: 0, }
    }

    /// Wrap the `DOMNode` in an additional set of attributes.
    ///
    /// Example:
    ///
    ///```rust
    /// use domafic::{DOMNode, empty};
    /// use domafic::tags::div;
    ///
    /// type MessageType = (); // Type of messages sent in response to JS events
    /// let my_div = div(empty::<MessageType>());
    /// let my_div_with_attrs = my_div.with_attributes([("key", "value")]);
    ///
    /// assert_eq!(my_div_with_attrs.get_attribute(0), Some(&("key", "value")));
    ///```
    fn with_attributes<A: AsRef<[KeyValue]>>(self, attrs: A) -> WithAttributes<Self, A> {
        WithAttributes { node: self, attributes: attrs, }
    }

    /// Process the listeners of the node, modifying the accumulator `acc`.
    ///
    /// If processing any listener fails, processing is short-circuited (the remaining listeners
    /// will not be processed) and `process_listeners` returns an error.
    fn process_listeners<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;

    /// Process the children of the node, modifying the accumulator `acc`.
    ///
    /// If processing any child fails, processing is short-circuited (the remaining children will
    /// not be processed) and `process_children` returns an error.
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;

    /// Returns an enum representing either the node's HTML tag or, in the case of a text node,
    /// the node's text value.
    fn value<'a>(&'a self) -> DOMValue<'a>;
}

/// A `KeyValue` is a pair of static strings corresponding to a mapping between a key and a value.
type KeyValue = (&'static str, &'static str);

/// "Value" of a `DOMNode`: either an element's tag name (e.g. "div"/"h1"/"body") or the text
/// value of a text node (e.g. "Hello world!").
pub enum DOMValue<'a> {
    /// Tag name for an element
    Element { tag: &'a str },

    /// The text value of a text node
    Text(&'a str),
}

/// Wrapper for `DOMNode`s that adds attributes.
pub struct WithAttributes<T: DOMNode, A: AsRef<[KeyValue]>> {
    node: T,
    attributes: A,
}

impl<T, A> DOMNode for WithAttributes<T, A> where T: DOMNode, A: AsRef<[KeyValue]> {
    type Message = T::Message;
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        let attributes = self.attributes.as_ref();
        attributes
            .get(index)
            .or_else(|| self.node.get_attribute(index - attributes.len()))
    }
    fn process_listeners<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.node.process_listeners::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { self.node.value() }
}


/// Iterator over the attributes of a `DOMNode`
pub struct AttributeIter<'a, T: DOMNode + 'a> {
    node: &'a T,
    index: usize,
}

impl<'a, T: DOMNode> Iterator for AttributeIter<'a, T> {
    type Item = &'a (&'static str, &'static str);
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.node.get_attribute(self.index);
        self.index += 1;
        res
    }
}

impl<'a, T: DOMNode> DOMNode for &'a T {
    type Message = T::Message;
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        (*self).get_attribute(index)
    }
    fn process_listeners<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        (*self).process_listeners::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        (*self).process_children::<P>(acc)
    }
    fn value<'b>(&'b self) -> DOMValue<'b> { (*self).value() }
}

trait IntoNode<M> {
    type Node: DOMNode<Message = M>;
    fn into_node(self) -> Self::Node;
}

#[cfg(any(feature = "use_std", test))]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct StringNode<Message>(String, PhantomData<Message>);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct StringRefNode<Message>(&'static str, PhantomData<Message>);

#[cfg(any(feature = "use_std", test))]
impl<M> IntoNode<M> for String {
    type Node = StringNode<M>;
    fn into_node(self) -> Self::Node {
        self.into()
    }
}

impl<M> IntoNode<M> for &'static str {
    type Node = StringRefNode<M>;
    fn into_node(self) -> Self::Node {
        self.into()
    }
}

#[cfg(any(feature = "use_std", test))]
impl<M> DOMNode for StringNode<M> {
    type Message = M;
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn process_listeners<P: ListenerProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(&self.0) }
}

impl<M> DOMNode for StringRefNode<M> {
    type Message = M;
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn process_listeners<P: ListenerProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(self.0) }
}

#[cfg(any(feature = "use_std", test))]
impl<Message> From<String> for StringNode<Message> {
    fn from(string: String) -> Self { StringNode(string, PhantomData) }
}

impl<Message> From<&'static str> for StringRefNode<Message> {
    fn from(string: &'static str) -> Self { StringRefNode(string, PhantomData) }
}

pub mod tags {
    use super::*;

    pub struct TagProperties<
        Children: DOMNodes,
        Attributes: AsRef<[KeyValue]>,
        Listens: Listeners<Message=Children::Message>>
        (Children, Attributes, Listens);

    type EmptyAttrs = [KeyValue; 0];

    impl<C: DOMNodes> From<C> for TagProperties<C, EmptyAttrs, EmptyListeners<C::Message>> {
        fn from(nodes: C) -> TagProperties<C, EmptyAttrs, EmptyListeners<C::Message>> {
            TagProperties (
                nodes,
                [],
                empty_listeners(),
            )
        }
    }

    pub fn attributes<A: AsRef<[KeyValue]>>(attrs: A) -> Attrs<A> {
        Attrs(attrs)
    }
    pub fn listeners<M, L: Listeners<Message=M>>(listeners: L) -> Listens<M, L> {
        Listens(listeners, PhantomData)
    }

    pub struct Attrs<A: AsRef<[KeyValue]>>(A);
    pub struct Listens<M, L: Listeners<Message=M>>(L, PhantomData<M>);

    impl<C: DOMNodes, A: AsRef<[KeyValue]>>
        From<(Attrs<A>, C)> for TagProperties<C, A, EmptyListeners<C::Message>>
    {
        fn from(props: (Attrs<A>, C)) -> TagProperties<C, A, EmptyListeners<C::Message>> {
            TagProperties (
                props.1,
                (props.0).0,
                empty_listeners(),
            )
        }
    }

    pub struct Tag<
        Contents: DOMNodes,
        Attributes: AsRef<[KeyValue]>,
        L: Listeners<Message=Contents::Message>>
    {
        tagname: &'static str,
        contents: Contents,
        attributes: Attributes,
        listeners: L,
    }

    impl<C: DOMNodes, A: AsRef<[KeyValue]>, L: Listeners<Message=C::Message>> DOMNode for Tag<C, A, L> {
        type Message = C::Message;
        fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
            self.attributes.as_ref().get(index)
        }
        fn process_listeners<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
            self.listeners.process_all::<P>(acc)
        }
        fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
            self.contents.process_all::<P>(acc)
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element {
                tag: self.tagname,
            }
        }
    }

    macro_rules! impl_tags {
        ($($tagname:ident),*) => { $(
            pub fn $tagname<
                C: DOMNodes,
                A: AsRef<[KeyValue]>,
                L: Listeners<Message=C::Message>,
                T: Into<TagProperties<C, A, L>>
                >(properties: T)
                -> Tag<C, A, L>
            {
                let TagProperties(contents, attributes, listeners) = properties.into();
                Tag {
                    tagname: stringify!($tagname),
                    contents: contents,
                    attributes: attributes,
                    listeners: listeners,
                }
            }
        )* }
    }

    impl_tags!(
        a, abbr, acronym, address, applet, area, article, aside, audio, b, base, basefont, bdi,
        bdo, big, blockquote, body, br, button, canvas, caption, center, cite, code, col, colgroup,
        datalist, dd, del, details, dfn, dialog, dir, div, dl, dt, em, embed, fieldset,
        figcaption, figure, font, footer, form, frame, framset, h1, h2, h3, h4, h5, h6, head,
        header, hr, i, iframe, img, input, ins, kbd, keygen, label, legend, li, link, main, map,
        mark, menu, menuitem, meta, meter, nav, noframes, noscript, object, ol, optgroup, option,
        output, p, param, pre, progress, q, rp, rt, ruby, s, samp, script, section, select, small,
        source, span, strike, strong, style, sub, summary, sup, table, tbody, td, textarea, tfoot,
        th, thead, time, title, tr, track, tt, u, ul, var, video, wbr
    );
}

// Note: without an extension to HTRBs, I don't know of a way to make the following traits generic
// enough to prevent duplication (need to be able to be generic on the `DOMNode`/`Listener` bounds)

/// `DOMNodeProcessor`s are used to iterate over `DOMNode`s which may or may not be the same type.
/// Implementations of this trait resemble traditional `fold` functions, modifying an accumulator
/// (of type `Acc`) and returning an error as necessary.
pub trait DOMNodeProcessor {

    /// Type of the accumulator updated by `get_processor`
    type Acc;

    /// Type of error returned by failed calls to `get_processor`
    type Error;

    /// Returns a folding function capable of processing elements of type `T: DOMNode`.
    ///
    /// TODO: Example
    fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error>;
}

pub trait DOMNodes {
    type Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

pub trait Listener {
    type Message;
    fn event_types_handled<'a>() -> &'a [events::EventType];
    fn handle_event(event: events::Event) -> Self::Message;
}

/// `ListenerProcessor`s are used to iterate over `Listeners`s which may or may not be the same
/// type. Implementations of this trait resemble traditional `fold` functions, modifying an
/// accumulator (of type `Acc`) and returning an error as necessary.
pub trait ListenerProcessor {

    /// Type of the accumulator updated by `get_processor`
    type Acc;

    /// Type of error returned by failed calls to `get_processor`
    type Error;

    /// Returns a folding function capable of processing elements of type `T: DOMNode`.
    ///
    /// TODO: Example
    fn get_processor<T: Listener>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error>;
}

pub trait Listeners {
    type Message;
    fn process_all<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;
}

pub mod events {
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
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmptyNodes<Message>(PhantomData<Message>);
pub fn empty<Message>() -> EmptyNodes<Message> { EmptyNodes(PhantomData) }
impl<M> DOMNodes for EmptyNodes<M> {
    type Message = M;
    fn process_all<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmptyListeners<Message>(PhantomData<Message>);
pub fn empty_listeners<Message>() -> EmptyListeners<Message> { EmptyListeners(PhantomData) }
impl<M> Listeners for EmptyListeners<M> {
    type Message = M;
    fn process_all<P: ListenerProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
}

impl<T: DOMNode> DOMNodes for T {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}

impl<T: Listener> Listeners for T {
    type Message = T::Message;
    fn process_all<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}

impl<T: DOMNodes> DOMNodes for Option<T> {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        if let Some(ref inner) = *self {
            inner.process_all::<P>(acc)
        } else {
            Ok(())
        }
    }
}

impl<L: Listeners> Listeners for Option<L> {
    type Message = L::Message;
    fn process_all<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        if let Some(ref inner) = *self {
            inner.process_all::<P>(acc)
        } else {
            Ok(())
        }
    }
}

impl<T: DOMNodes> DOMNodes for [T] {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

impl<T: Listeners> Listeners for [T] {
    type Message = T::Message;
    fn process_all<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "use_std", test))]
impl<T: DOMNodes> DOMNodes for Vec<T> {
    type Message = T::Message;
    fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "use_std", test))]
impl<T: Listeners> Listeners for Vec<T> {
    type Message = T::Message;
    fn process_all<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        for x in self {
            x.process_all::<P>(acc)?;
        }
        Ok(())
    }
}

macro_rules! array_impls {
    ($($len:expr,)*) => { $(
        impl<T: DOMNodes> DOMNodes for [T; $len] {
            type Message = T::Message;
            fn process_all<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
                for x in self {
                    x.process_all::<P>(acc)?;
                }
                Ok(())
            }
        }

        impl<T: Listeners> Listeners for [T; $len] {
            type Message = T::Message;
            fn process_all<P: ListenerProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
                for x in self {
                    x.process_all::<P>(acc)?;
                }
                Ok(())
            }
        }
    )* }
}

array_impls!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
);

// Credit to @shepmaster for structure of recursive tuple macro
macro_rules! tuple_impls {
    () => {};

    // Copywrite @shepmaster
    (($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
        /*
         * Invoke recursive reversal of list that ends in the macro expansion implementation
         * of the reversed list
        */
        tuple_impls!([($idx, $typ);] $( ($nidx => $ntyp), )*);
        tuple_impls!($( ($nidx => $ntyp), )*); // invoke macro on tail
    };

    /*
     * ([accumulatedList], listToReverse); recursively calls tuple_impls until the list to reverse
     + is empty (see next pattern)
    */
    ([$(($accIdx: tt, $accTyp: ident);)+]  ($idx:tt => $typ:ident), $( ($nidx:tt => $ntyp:ident), )*) => {
      tuple_impls!([($idx, $typ); $(($accIdx, $accTyp); )*] $( ($nidx => $ntyp), ) *);
    };

    // Finally expand into the implementation
    ([($idx:tt, $typ:ident); $( ($nidx:tt, $ntyp:ident); )*]) => {
        impl<$typ, $( $ntyp ),*> DOMNodes for ($typ, $( $ntyp ),*)
            where $typ: DOMNodes,
                  $( $ntyp: DOMNodes<Message=$typ::Message>),*
        {
            type Message = $typ::Message;
            fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: DOMNodeProcessor {
                &self.$idx.process_all::<P>(acc)?;
                $(
                    &self.$nidx.process_all::<P>(acc)?;
                )*
                Ok(())
            }
        }

        impl<$typ, $( $ntyp ),*> Listeners for ($typ, $( $ntyp ),*)
            where $typ: Listeners,
                  $( $ntyp: Listeners<Message=$typ::Message>),*
        {
            type Message = $typ::Message;
            fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                    where P: ListenerProcessor {
                &self.$idx.process_all::<P>(acc)?;
                $(
                    &self.$nidx.process_all::<P>(acc)?;
                )*
                Ok(())
            }
        }
    }
}

tuple_impls!(
    (9 => J),
    (8 => I),
    (7 => H),
    (6 => G),
    (5 => F),
    (4 => E),
    (3 => D),
    (2 => C),
    (1 => B),
    (0 => A),
);

#[cfg(feature = "use_either_n")]
mod either_impls {
    use super::{DOMNodes, DOMNodeProcessor, Listeners, ListenerProcessor};

    extern crate either_n;
    use self::either_n::*;

    macro_rules! impl_enums {
        () => {};

        (($enum_name_head:ident, $n_head:ident),
        $(($enum_name_tail:ident, $n_tail:ident),)*) => {

            impl<$n_head, $( $n_tail ),*> DOMNodes for
                $enum_name_head<$n_head, $( $n_tail ),*>
                where $n_head: DOMNodes, $( $n_tail: DOMNodes<Message=$n_head::Message> ),*
            {
                type Message = $n_head::Message;
                fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                        where P: DOMNodeProcessor {
                    match *self {
                        $enum_name_head::$n_head(ref value) =>
                            value.process_all::<P>(acc)?,
                        $(
                            $enum_name_head::$n_tail(ref value) =>
                                value.process_all::<P>(acc)?
                        ),*
                    };
                    Ok(())
                }
            }

            impl<$n_head, $( $n_tail ),*> Listeners for
                $enum_name_head<$n_head, $( $n_tail ),*>
                where $n_head: Listeners, $( $n_tail: Listeners<Message=$n_head::Message> ),*
            {
                type Message = $n_head::Message;
                fn process_all<P>(&self, acc: &mut P::Acc) -> Result<(), P::Error>
                        where P: ListenerProcessor {
                    match *self {
                        $enum_name_head::$n_head(ref value) =>
                            value.process_all::<P>(acc)?,
                        $(
                            $enum_name_head::$n_tail(ref value) =>
                                value.process_all::<P>(acc)?
                        ),*
                    };
                    Ok(())
                }
            }

            impl_enums!($( ($enum_name_tail, $n_tail), )*);
        }
    }

    impl_enums!(
        (Either8, Eight),
        (Either7, Seven),
        (Either6, Six),
        (Either5, Five),
        (Either4, Four),
        (Either3, Three),
        (Either2, Two),
        (Either1, One),
    );
}

#[cfg(any(feature = "use_std", test))]
pub mod html_writer {
    use super::{DOMNode, DOMNodeProcessor, DOMValue};
    use std::marker::PhantomData;
    use std::io;

    pub struct HtmlWriter<W: io::Write>(PhantomData<W>);
    impl<W: io::Write> DOMNodeProcessor for HtmlWriter<W> {
        type Acc = W;
        type Error = io::Error;

        fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Self::Error> {
            fn add_node<W, T>(w: &mut W, node: &T) -> Result<(), io::Error>
                    where W: io::Write, T: DOMNode {
                match node.value() {
                    DOMValue::Element { tag: tagname } => {
                        write!(w, "<{}", tagname)?;
                        for attr in node.attributes() {
                            write!(w, " {}=\"{}\"", attr.0, attr.1)?;
                        }
                        write!(w, ">")?;
                        node.process_children::<HtmlWriter<W>>(w)?;
                        write!(w, "</{}>", tagname)?;
                    }
                    DOMValue::Text(text) => {
                        // TODO: HTML escaping
                        write!(w, "{}", text)?;
                    }
                }
                Ok(())
            }
            add_node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::tags::*;
    use super::html_writer::*;

    #[cfg(feature = "use_either_n")]
    extern crate either_n;
    #[cfg(feature = "use_either_n")]
    use self::either_n::*;

    struct BogusOne;
    impl DOMNode for BogusOne {
        type Message = Never;
        fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
        fn process_listeners<P: ListenerProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_one" }
        }
    }

    struct BogusTwo;
    impl DOMNode for BogusTwo {
        type Message = Never;
        fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
        fn process_listeners<P: ListenerProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
            Ok(())
        }
        fn value<'a>(&'a self) -> DOMValue<'a> {
            DOMValue::Element { tag: "bogus_tag_two" }
        }
    }

    struct ChildCounter;
    #[derive(Debug, Clone, Copy)]
    enum Never {}
    impl DOMNodeProcessor for ChildCounter {
        type Acc = usize;
        type Error = Never;

        fn get_processor<T: DOMNode>() -> fn(&mut Self::Acc, &T) -> Result<(), Never> {
            fn incr<T: DOMNode>(count: &mut usize, _node: &T) -> Result<(), Never> {
                *count += 1;
                Ok(())
            }
            incr
        }
    }

    fn html_sample() -> impl DOMNode<Message = Never> {
        div ((
            attributes([("attr", "value")]),
            (
            BogusOne,
            BogusOne,
            BogusTwo,
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
        (BogusOne, &BogusOne, &BogusTwo).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        (BogusOne, (BogusOne,), BogusOne).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        [BogusOne, BogusOne, BogusOne].process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(3, count);

        count = 0;
        (BogusOne, BogusOne,
            [BogusOne, BogusOne, BogusOne],
            [(BogusOne)],
            vec![empty(), empty(), empty()],
            [&BogusTwo, &BogusTwo, &BogusTwo],
        ).process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(9, count);

        let sample = html_sample();

        count = 0;
        sample.process_all::<ChildCounter>(&mut count).unwrap();
        assert_eq!(1, count);

        count = 0;
        sample.process_children::<ChildCounter>(&mut count).unwrap();
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
        assert_eq!(div.get_attribute(0), Some(&("attr1", "val1")));
        assert_eq!(div.get_attribute(1), Some(&("attr2", "val2")));
        assert_eq!(div.get_attribute(2), Some(&("attr3", "val3")));
        assert_eq!(div.get_attribute(3), None);

        let mut attr_iter = div.attributes();
        assert_eq!(attr_iter.next(), Some(&("attr1", "val1")));
        assert_eq!(attr_iter.next(), Some(&("attr2", "val2")));
        assert_eq!(attr_iter.next(), Some(&("attr3", "val3")));
        assert_eq!(attr_iter.next(), None);
    }

    #[test]
    fn builds_attribute_list() {
        let div1 = div(empty::<Never>())
            .with_attributes([("attr2", "val2"), ("attr3", "val3")])
            .with_attributes([("attr1", "val1")]);
        check_attribute_list(div1);

        let div2 = div((
            attributes([("attr2", "val2"), ("attr3", "val3")]),
            div(empty::<Never>())
        )).with_attributes([("attr1", "val1")]);
        check_attribute_list(div2);
    }
}
