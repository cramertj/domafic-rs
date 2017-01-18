use processors::{DomNodes, Listeners};
use KeyValue;
use empty::{empty, empty_listeners, EmptyNodes, EmptyListeners};

/// A `DomNode` specifies the HTML DOM (Document Object Model) representation of a type.
///
/// Note that there can be many different types that map to the same HTML. For example, both
/// `String` and `str` can be used to create HTML text nodes.
pub trait DomNode: Sized {

    /// The type of message sent by a listener. Messages of this type should be used to update
    /// application state.
    type Message;

    /// The type of the set of children contained by the `DomNode`.
    ///
    /// Examples:
    /// `Tag<...>`
    /// `(Tag<...>, Tag<...>, Tag<...>)`
    /// `[Tag<...>; 5]`
    type Children: DomNodes<Message=Self::Message>;

    /// The type of the set of listeners watching this `DomNode` for events.
    ///
    /// Examples:
    /// `FnListener<...>`
    /// `(FnListener<..>, FnListener<...>)`
    /// `[Box<Listener<Message=()>>; 5]`
    type Listeners: Listeners<Message=Self::Message>;

    /// The type of the `DomNode` with its listeners replaced by `EmptyListeners`.
    ///
    /// This is useful for splitting the `DomNode` up into its listener and non-listener components
    /// so that they can be transformed separately.
    type WithoutListeners:
        DomNode<
            Message=Self::Message,
            Children=Self::Children,
            Listeners=EmptyListeners<Self::Message>
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
    /// use domafic::{DomNode, KeyIter, IntoNode};
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
    ///         div(index.to_string().into_node())
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
    fn with_key(self, key: usize) -> WithKey<Self> {
        assert!(self.key() == None, "Attempted to add multiple keys to a DomNode");
        WithKey(self, key as u32)
    }

    /// Returns a wrapper that can displayed as HTML
    #[cfg(feature = "use_std")]
    fn displayable(&self) -> ::html_writer::HtmlDisplayable<Self> {
        ::html_writer::HtmlDisplayable(self)
    }

    /// Get the nth attribute for a given `DomNode`.
    ///
    /// If `node.get_attribute(i)` returns `None`, `node.get_attribute(j)` should return `None`
    /// for all `j >= i`.
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue>;

    /// Returns an iterator over a `DomNode`'s attributes.
    fn attributes(&self) -> AttributeIter<Self> {
        AttributeIter { node: self, index: 0, }
    }

    /// Wrap the `DomNode` in an additional set of attributes.
    ///
    /// Example:
    ///
    ///```rust
    /// use domafic::DomNode;
    /// use domafic::empty::empty;
    /// use domafic::tags::div;
    /// use domafic::AttributeValue::Str;
    ///
    /// type MessageType = (); // Type of messages sent in response to JS events
    ///
    /// // Need to manually specify message type here since it can't be inferred
    /// let my_div = div(empty::<MessageType>());
    ///
    /// let my_div_with_attrs = my_div.with_attributes([("key", Str("value"))]);
    ///
    /// assert_eq!(my_div_with_attrs.get_attribute(0), Some(&("key", Str("value"))));
    ///```
    fn with_attributes<A: AsRef<[KeyValue]>>(self, attributes: A) -> WithAttributes<Self, A> {
        WithAttributes { node: self, attributes: attributes, }
    }

    /// Wrap the `DomNode` in an additional set of liseners.
    ///
    /// Example:
    ///
    ///```rust
    /// use domafic::DomNode;
    /// use domafic::empty::empty;
    /// use domafic::listener::on;
    /// use domafic::tags::div;
    /// use domafic::AttributeValue::Str;
    ///
    /// struct Clicked; // Type of messages sent
    ///
    /// // We don't need to manually annotate the message type here since it can be inferred
    /// let my_div = div(());
    ///
    /// let _my_div_with_listener = my_div.with_listeners(on("click", |_| Clicked));
    ///```
    fn with_listeners<L: Listeners<Message=Self::Message>>(self, listeners: L) ->
            WithListeners<Self::WithoutListeners, (L, Self::Listeners)> {
        let (without_listeners, old_listeners) = self.split_listeners();
        WithListeners { node: without_listeners, listeners: (listeners, old_listeners), }
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
pub struct WithKey<T: DomNode>(T, u32);
impl<T: DomNode> DomNode for WithKey<T> {
    type Message = T::Message;
    type Children = T::Children;
    type Listeners = T::Listeners;
    type WithoutListeners = WithKey<T::WithoutListeners>;

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
        (WithKey(node, self.1), listeners)
    }
    fn value(&self) -> DomValue { self.0.value() }
}

/// Wrapper for `DomNode`s that adds attributes.
pub struct WithAttributes<T: DomNode, A: AsRef<[KeyValue]>> {
    node: T,
    attributes: A,
}

impl<T, A> DomNode for WithAttributes<T, A> where T: DomNode, A: AsRef<[KeyValue]> {
    type Message = T::Message;
    type Children = T::Children;
    type Listeners = T::Listeners;
    type WithoutListeners = WithAttributes<T::WithoutListeners, A>;
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
            },
            listeners
        )
    }
    fn value(&self) -> DomValue { self.node.value() }
}

/// Wrapper for `DomNode`s that adds listeners.
pub struct WithListeners<T: DomNode<Message=L::Message, Listeners=EmptyListeners<L::Message>>, L: Listeners> {
    node: T,
    listeners: L,
}

impl<T, L> DomNode for WithListeners<T, L>
    where T: DomNode<Message=L::Message, Listeners=EmptyListeners<L::Message>>, L: Listeners
{
    type Message = L::Message;
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
pub struct AttributeIter<'a, T: DomNode + 'a> {
    node: &'a T,
    index: usize,
}

impl<'a, T: DomNode> Iterator for AttributeIter<'a, T> {
    type Item = &'a KeyValue;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.node.get_attribute(self.index);
        self.index += 1;
        res
    }
}

/// Types that can be converted into `DomNode`s with messages of type `M`
pub trait IntoNode<M> {
    /// The type of the resulting node
    type Node: DomNode<Message = M>;
    /// Consume `self` to produce a `DomNode`
    fn into_node(self) -> Self::Node;
}

#[cfg(any(feature = "use_std", test))]
/// `DomNode` wrapper for `String`s
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StringNode<Message>(String, EmptyNodes<Message>, EmptyListeners<Message>);

/// `DomNode` wrapper for `&'static str`s
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StringRefNode<'a, Message>(&'a str, EmptyNodes<Message>, EmptyListeners<Message>);

#[cfg(any(feature = "use_std", test))]
impl<M> IntoNode<M> for String {
    type Node = StringNode<M>;
    fn into_node(self) -> Self::Node {
        self.into()
    }
}

impl<'a, M> IntoNode<M> for &'a str {
    type Node = StringRefNode<'a, M>;
    fn into_node(self) -> Self::Node {
        self.into()
    }
}

#[cfg(any(feature = "use_std", test))]
impl<M> DomNode for StringNode<M> {
    type Message = M;
    type Children = EmptyNodes<M>;
    type Listeners = EmptyListeners<M>;
    type WithoutListeners = Self;
    fn key(&self) -> Option<u32> { None }
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn children(&self) -> &Self::Children {
        &self.1
    }
    fn listeners(&self) -> &Self::Listeners {
        &self.2
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        (&self.1, &self.2)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        (self, empty_listeners())
    }
    fn value(&self) -> DomValue { DomValue::Text(&self.0) }
}

impl<'a, M> DomNode for StringRefNode<'a, M> {
    type Message = M;
    type Children = EmptyNodes<M>;
    type Listeners = EmptyListeners<M>;
    type WithoutListeners = Self;
    fn key(&self) -> Option<u32> { None }
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn children(&self) -> &Self::Children {
        &self.1
    }
    fn listeners(&self) -> &Self::Listeners {
        &self.2
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        (&self.1, &self.2)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        (self, empty_listeners())
    }
    fn value(&self) -> DomValue { DomValue::Text(self.0) }
}

#[cfg(any(feature = "use_std", test))]
impl<Message> From<String> for StringNode<Message> {
    fn from(string: String) -> Self { StringNode(string, empty(), empty_listeners()) }
}

impl<'a, Message> From<&'a str> for StringRefNode<'a, Message> {
    fn from(string: &'a str) -> Self { StringRefNode(string, empty(), empty_listeners()) }
}
