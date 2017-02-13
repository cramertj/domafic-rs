use processors::{DomNodes, DomNodeProcessor, Listeners, EmptyListeners};
use KeyValue;

use opt_std::marker::PhantomData;

/// A `DomNode` specifies the HTML DOM (Document Object Model) representation of a type.
///
/// Note that there can be many different types that map to the same HTML. For example, both
/// `String` and `str` can be used to create HTML text nodes.
pub trait DomNode<Message>: DomNodes<Message> + Sized {

    /// The type of the set of children contained by the `DomNode`.
    ///
    /// Examples:
    /// `Tag<...>`
    /// `(Tag<...>, Tag<...>, Tag<...>)`
    /// `[Tag<...>; 5]`
    type Children: DomNodes<Message>;

    /// The type of the set of listeners watching this `DomNode` for events.
    ///
    /// Examples:
    /// `FnListener<...>`
    /// `(FnListener<..>, FnListener<...>)`
    /// `[Box<Listener<Message=()>>; 5]`
    type Listeners: Listeners<Message>;

    /// The type of the `DomNode` with its listeners replaced by `EmptyListeners`.
    ///
    /// This is useful for splitting the `DomNode` up into its listener and non-listener components
    /// so that they can be transformed separately.
    type WithoutListeners:
        DomNode<
            Message,
            Children=Self::Children,
            Listeners=EmptyListeners
            >;

    /// If present, the key will be included in the `KeyStack` returned alongside a message.
    /// This should be used to differentiate messages from peer `DomNode`s.
    fn key(&self) -> Option<u32>;

    /// Add a key to this `DomNode`. This method will panic if the node already has a key.
    ///
    /// Keys are used to differentiate between large numbers of similar components.
    /// When an event occurs in a keyed component, the keys of that component and all of its
    /// parent components will be returned to the ""
    ///
    /// Example:
    ///
    /// ```rust
    /// use domafic::{DomNode, KeyIter};
    /// use domafic::tags::div;
    /// use domafic::listener::on;
    ///
    /// #[cfg(target_os = "emscripten")]
    /// use domafic::web_render::run;
    ///
    /// struct Clicked;
    /// type State = ();
    ///
    /// let _render = |_state: &State| div(
    ///     (0..50).into_iter().map(|index|
    ///         div(index.to_string())
    ///             .with_listeners(on("click", |_| Clicked))
    ///             .with_key(index)
    ///     ).collect::<Vec<_>>()
    /// );
    ///
    /// let _update = |_state: &mut State, _msg: Clicked, mut keys: KeyIter|
    ///    println!("div number {} was clicked", keys.next().unwrap());
    ///
    /// // If using in a browser:
    /// #[cfg(target_os = "emscripten")]
    /// run("body", _update, _render, ());
    /// ```
    fn with_key(self, key: usize) -> WithKey<Message, Self> {
        assert!(self.key() == None, "Attempted to add multiple keys to a DomNode");
        WithKey(self, key as u32, PhantomData)
    }

    /// Returns a wrapper that can displayed as HTML
    #[cfg(feature = "use_std")]
    fn displayable(&self) -> ::html_writer::HtmlDisplayable<Message, Self> {
        ::html_writer::HtmlDisplayable(self, PhantomData)
    }

    /// Get the nth attribute for a given `DomNode`.
    ///
    /// If `node.get_attribute(i)` returns `None`, `node.get_attribute(j)` should return `None`
    /// for all `j >= i`.
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue>;

    /// Returns an iterator over a `DomNode`'s attributes.
    fn attributes(&self) -> AttributeIter<Message, Self> {
        AttributeIter { node: self, index: 0, _marker: PhantomData }
    }

    /// Wrap the `DomNode` in an additional set of attributes.
    ///
    /// Example:
    ///
    ///```rust
    /// use domafic::DomNode;
    /// use domafic::tags::div;
    /// use domafic::AttributeValue::Str;
    /// use std::marker::PhantomData;
    ///
    /// type MessageType = (); // Type of messages sent in response to JS events
    ///
    /// // Need to manually specify message type here since it can't be inferred
    /// let my_div = div(PhantomData::<MessageType>);
    ///
    /// let my_div_with_attrs = my_div.with_attributes([("key", Str("value"))]);
    ///
    /// assert_eq!(my_div_with_attrs.get_attribute(0), Some(&("key", Str("value"))));
    ///```
    fn with_attributes<A: AsRef<[KeyValue]>>(self, attributes: A) -> WithAttributes<Message, Self, A> {
        WithAttributes { node: self, attributes: attributes, _marker: PhantomData }
    }

    /// Wrap the `DomNode` in an additional set of liseners.
    ///
    /// Example:
    ///
    ///```rust
    /// use domafic::DomNode;
    /// use domafic::listener::on;
    /// use domafic::tags::div;
    ///
    /// struct Clicked; // Type of messages sent
    ///
    /// // We don't need to manually annotate the message type here since it can be inferred
    /// let my_div = div(());
    ///
    /// let _my_div_with_listener = my_div.with_listeners(on("click", |_| Clicked));
    ///```
    fn with_listeners<L: Listeners<Message>>(self, listeners: L) ->
            WithListeners<Message, Self::WithoutListeners, (L, Self::Listeners)> {
        let (without_listeners, old_listeners) = self.split_listeners();
        WithListeners {
            node: without_listeners,
            listeners: (listeners, old_listeners),
            _marker: PhantomData,
        }
    }

    // TODO once type ATCs land
    // type Mapped<Mapper: Map<In=Self::Message>>: DomNode<Message=Mapper::Out>
    // fn map_listeners<Mapper: Map<In=Self::Message>>(self) -> Mapped<Mapper>

    /// Returns a reference to the children of this `DomNode`
    fn children(&self) -> &Self::Children;

    /// Returns a reference to the listeners listening for events on this `DomNode`
    fn listeners(&self) -> &Self::Listeners;

    /// Returns a reference to both the children and listeners of this `DomNode`
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners);

    /// Splits `self` into two separate components, one with and one without listeners.
    ///
    /// This is used to perform type-level modifications to the listeners.
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners);

    /// Returns an enum representing either the node's HTML tag or, in the case of a text node,
    /// the node's text value.
    fn value(&self) -> DomValue;

    /// Writes the `DomNode`'s HTML representation to `writer`.
    #[cfg(any(feature = "use_std", test))]
    fn write_html<W: ::std::io::Write>(&self, writer: &mut W) -> ::std::io::Result<()> {
        use html_writer::HtmlWriter;
        self.process_all::<HtmlWriter<W>>(writer)
    }
}

/// "Value" of a `DomNode`: either an element's tag name (e.g. "div"/"h1"/"body") or the text
/// value of a text node (e.g. "Hello world!").
pub enum DomValue<'a> {
    /// A tag element
    Element {
        /// `&'static str` tag name, such as `div` or `span`.
        tag: &'static str
    },

    /// A text node
    Text(&'a str),
}

/// A `DomNode` with a key
pub struct WithKey<M, T: DomNode<M>>(T, u32, PhantomData<M>);
impl<M, T: DomNode<M>> DomNodes<M> for WithKey<M, T> {
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}
impl<M, T: DomNode<M>> DomNode<M> for WithKey<M, T> {
    type Children = T::Children;
    type Listeners = T::Listeners;
    type WithoutListeners = WithKey<M, T::WithoutListeners>;

    fn key(&self) -> Option<u32> { Some(self.1) }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.0.get_attribute(index)
    }
    fn children(&self) -> &Self::Children {
        self.0.children()
    }
    fn listeners(&self) -> &Self::Listeners {
        self.0.listeners()
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        self.0.children_and_listeners()
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        let (node, listeners) = self.0.split_listeners();
        (WithKey(node, self.1, PhantomData), listeners)
    }
    fn value(&self) -> DomValue { self.0.value() }
}

/// Wrapper for `DomNode`s that adds attributes.
pub struct WithAttributes<M, T: DomNode<M>, A: AsRef<[KeyValue]>> {
    node: T,
    attributes: A,
    _marker: PhantomData<M>
}
impl<M, T: DomNode<M>, A: AsRef<[KeyValue]>> DomNodes<M> for WithAttributes<M, T, A> {
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}
impl<M, T, A> DomNode<M> for WithAttributes<M, T, A> where T: DomNode<M>, A: AsRef<[KeyValue]> {
    type Children = T::Children;
    type Listeners = T::Listeners;
    type WithoutListeners = WithAttributes<M, T::WithoutListeners, A>;
    fn key(&self) -> Option<u32> { self.node.key() }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        let attributes = self.attributes.as_ref();
        attributes
            .get(index)
            .or_else(|| self.node.get_attribute(index - attributes.len()))
    }
    fn children(&self) -> &Self::Children {
        self.node.children()
    }
    fn listeners(&self) -> &Self::Listeners {
        self.node.listeners()
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        self.node.children_and_listeners()
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        let (node, listeners) = self.node.split_listeners();
        (
            WithAttributes {
                node: node,
                attributes: self.attributes,
                _marker: PhantomData,
            },
            listeners
        )
    }
    fn value(&self) -> DomValue { self.node.value() }
}

/// Wrapper for `DomNode`s that adds listeners.
pub struct WithListeners<M, T: DomNode<M, Listeners=EmptyListeners>, L: Listeners<M>> {
    node: T,
    listeners: L,
    _marker: PhantomData<M>,
}
impl<M, T: DomNode<M, Listeners=EmptyListeners>, L: Listeners<M>> DomNodes<M> for WithListeners<M, T, L> {
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}
impl<M, T, L> DomNode<M> for WithListeners<M, T, L>
    where T: DomNode<M, Listeners=EmptyListeners>, L: Listeners<M>
{
    type Children = T::Children;
    type Listeners = L;
    type WithoutListeners = T;
    fn key(&self) -> Option<u32> { self.node.key() }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.node.get_attribute(index)
    }
    fn children(&self) -> &Self::Children {
        self.node.children()
    }
    fn listeners(&self) -> &Self::Listeners {
        &self.listeners
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        (self.node.children(), &self.listeners)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        (self.node, self.listeners)
    }
    fn value(&self) -> DomValue { self.node.value() }
}

/// Iterator over the attributes of a `DomNode`
pub struct AttributeIter<'a, M, T: DomNode<M> + 'a> {
    node: &'a T,
    index: usize,
    _marker: PhantomData<M>,
}

impl<'a, M, T: DomNode<M>> Iterator for AttributeIter<'a, M, T> {
    type Item = &'a KeyValue;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.node.get_attribute(self.index);
        self.index += 1;
        res
    }
}

static EMPTY_NODES_REF: &'static () = &();
static EMPTY_LISTN_REF: &'static EmptyListeners = &EmptyListeners;

#[cfg(any(feature = "use_std", test))]
impl<M> DomNodes<M> for String {
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}
#[cfg(any(feature = "use_std", test))]
impl<M> DomNode<M> for String {
    type Children = ();
    type Listeners = EmptyListeners;
    type WithoutListeners = String;
    fn key(&self) -> Option<u32> { None }
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> {
        None
    }
    fn children(&self) -> &Self::Children {
        EMPTY_NODES_REF
    }
    fn listeners(&self) -> &Self::Listeners {
        EMPTY_LISTN_REF
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        (EMPTY_NODES_REF, EMPTY_LISTN_REF)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        (self, EmptyListeners)
    }
    fn value(&self) -> DomValue { DomValue::Text(&self) }
}

impl<'b, M> DomNodes<M> for &'b str {
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}
impl<'a, M> DomNode<M> for &'a str {
    type Children = ();
    type Listeners = EmptyListeners;
    type WithoutListeners = Self;
    fn key(&self) -> Option<u32> { None }
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn children(&self) -> &Self::Children {
        EMPTY_NODES_REF
    }
    fn listeners(&self) -> &Self::Listeners {
        EMPTY_LISTN_REF
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        (EMPTY_NODES_REF, EMPTY_LISTN_REF)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        (self, EmptyListeners)
    }
    fn value(&self) -> DomValue { DomValue::Text(self) }
}
