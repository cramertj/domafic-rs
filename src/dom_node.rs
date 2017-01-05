use processors::{DOMNodes, Listeners};
use KeyValue;
use empty::{empty, empty_listeners, EmptyNodes, EmptyListeners};

/// A `DOMNode` specifies the HTML DOM (Document Object Model) representation of a type.
///
/// Note that there can be many different types that map to the same HTML. For example, both
/// `String` and `str` can be used to create HTML text nodes.
pub trait DOMNode: Sized {

    /// Type of message sent by a listener. Messages of this type should be used to update
    /// application state.
    type Message;
    type Children: DOMNodes<Message=Self::Message>;
    type Listeners: Listeners<Message=Self::Message>;
    type WithoutListeners:
        DOMNode<
            Message=Self::Message,
            Children=Self::Children,
            Listeners=EmptyListeners<Self::Message>
            >;

    /// If present, the key will be included in the `KeyStack` returned alongside a message.
    /// This should be used to differentiate messages from peer `DOMNode`s.
    fn key(&self) -> Option<u32>;

    // TODO fix u32/usize crud by sending a usize to emscripten via casting to and from a pointer
    /// Note: currently accepts only 32-bit keys. `usize` input is provided for convenience of use with
    /// methods like `Iterator::enumerate` which provide a usize.
    fn with_key(self, key: usize) -> WithKey<Self> {
        assert!(self.key() == None, "Attempted to add multiple keys to a DOMNode");
        WithKey(self, key as u32)
    }

    /// Returns a type that can be displayed as HTML
    #[cfg(feature = "use_std")]
    fn displayable<'a>(&'a self) -> ::html_writer::HtmlDisplayable<'a, Self> {
        ::html_writer::HtmlDisplayable(self)
    }

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
    /// use domafic::DOMNode;
    /// use domafic::empty::empty;
    /// use domafic::tags::div;
    ///
    /// type MessageType = (); // Type of messages sent in response to JS events
    /// let my_div = div(empty::<MessageType>());
    /// let my_div_with_attrs = my_div.with_attributes([("key", "value")]);
    ///
    /// assert_eq!(my_div_with_attrs.get_attribute(0), Some(&("key", "value")));
    ///```
    fn with_attributes<A: AsRef<[KeyValue]>>(self, attributes: A) -> WithAttributes<Self, A> {
        WithAttributes { node: self, attributes: attributes, }
    }

    fn with_listeners<L: Listeners<Message=Self::Message>>(self, listeners: L) ->
            WithListeners<Self::WithoutListeners, (L, Self::Listeners)> {
        let (without_listeners, old_listeners) = self.split_listeners();
        WithListeners { node: without_listeners, listeners: (listeners, old_listeners), }
    }

    // TODO once type ATCs land
    // type Mapped<Mapper: Map<In=Self::Message>>: DOMNode<Message=Mapper::Out>
    // fn map_listeners<Mapper: Map<In=Self::Message>>(self) -> Mapped<Mapper>

    fn children(&self) -> &Self::Children;
    fn listeners(&self) -> &Self::Listeners;
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners);
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners);

    /// Returns an enum representing either the node's HTML tag or, in the case of a text node,
    /// the node's text value.
    fn value<'a>(&'a self) -> DOMValue<'a>;
}

/// "Value" of a `DOMNode`: either an element's tag name (e.g. "div"/"h1"/"body") or the text
/// value of a text node (e.g. "Hello world!").
pub enum DOMValue<'a> {
    /// Tag name for an element
    Element { tag: &'static str },

    /// The text value of a text node
    Text(&'a str),
}

pub struct WithKey<T: DOMNode>(T, u32);
impl<T: DOMNode> DOMNode for WithKey<T> {
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
    fn value<'a>(&'a self) -> DOMValue<'a> { self.0.value() }
}

/// Wrapper for `DOMNode`s that adds attributes.
pub struct WithAttributes<T: DOMNode, A: AsRef<[KeyValue]>> {
    node: T,
    attributes: A,
}

impl<T, A> DOMNode for WithAttributes<T, A> where T: DOMNode, A: AsRef<[KeyValue]> {
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
    fn value<'a>(&'a self) -> DOMValue<'a> { self.node.value() }
}

/// Wrapper for `DOMNode`s that adds listeners.
pub struct WithListeners<T: DOMNode<Message=L::Message, Listeners=EmptyListeners<L::Message>>, L: Listeners> {
    node: T,
    listeners: L,
}

impl<T, L> DOMNode for WithListeners<T, L>
    where T: DOMNode<Message=L::Message, Listeners=EmptyListeners<L::Message>>, L: Listeners
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
        (&self.node.children(), &self.listeners)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        (self.node, self.listeners)
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

pub trait IntoNode<M> {
    type Node: DOMNode<Message = M>;
    fn into_node(self) -> Self::Node;
}

#[cfg(any(feature = "use_std", test))]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StringNode<Message>(String, EmptyNodes<Message>, EmptyListeners<Message>);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StringRefNode<Message>(&'static str, EmptyNodes<Message>, EmptyListeners<Message>);

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
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(&self.0) }
}

impl<M> DOMNode for StringRefNode<M> {
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
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(self.0) }
}

#[cfg(any(feature = "use_std", test))]
impl<Message> From<String> for StringNode<Message> {
    fn from(string: String) -> Self { StringNode(string, empty(), empty_listeners()) }
}

impl<Message> From<&'static str> for StringRefNode<Message> {
    fn from(string: &'static str) -> Self { StringRefNode(string, empty(), empty_listeners()) }
}
