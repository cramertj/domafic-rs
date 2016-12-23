use processors::{DOMNodeProcessor, ListenerProcessor};
use {Listeners, KeyValue};
use listener::{Map, MappedListenerProcessor};

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

    /// If present, the key will be included in the `KeyStack` returned alongside a message.
    /// This should be used to differentiate messages from peer `DOMNode`s.
    fn key(&self) -> Option<usize>;

    fn with_key(self, key: usize) -> WithKey<Self> {
        assert!(self.key() == None, "Attempted to add multiple keys to a DOMNode");
        WithKey(self, key)
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

    fn with_listeners<L: Listeners>(self, listeners: L) -> WithListeners<Self, L> {
        WithListeners { node: self, listeners: listeners, }
    }

    fn map_listeners<NewMessage, M: Map<Self::Message, Out=NewMessage>>(self)
        -> WithMessageMap<Self, NewMessage, M>
    {
        WithMessageMap(self, PhantomData)
    }

    /// Process the listeners of the node, modifying the accumulator `acc`.
    ///
    /// If processing any listener fails, processing is short-circuited (the remaining listeners
    /// will not be processed) and `process_listeners` returns an error.
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;

    /// Process the children of the node, modifying the accumulator `acc`.
    ///
    /// If processing any child fails, processing is short-circuited (the remaining children will
    /// not be processed) and `process_children` returns an error.
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error>;

    /// Returns an enum representing either the node's HTML tag or, in the case of a text node,
    /// the node's text value.
    fn value<'a>(&'a self) -> DOMValue<'a>;
}

/// "Value" of a `DOMNode`: either an element's tag name (e.g. "div"/"h1"/"body") or the text
/// value of a text node (e.g. "Hello world!").
pub enum DOMValue<'a> {
    /// Tag name for an element
    Element { tag: &'a str },

    /// The text value of a text node
    Text(&'a str),
}

pub struct WithKey<T: DOMNode>(T, usize);
impl<T: DOMNode> DOMNode for WithKey<T> {
    type Message = T::Message;
    fn key(&self) -> Option<usize> { Some(self.1) }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.0.get_attribute(index)
    }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.0.process_listeners::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.0.process_children::<P>(acc)
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { self.0.value() }
}

pub struct WithMessageMap<T: DOMNode, NewMessage, M: Map<T::Message, Out=NewMessage>>
    (T, PhantomData<(NewMessage, M)>);

impl<T: DOMNode, NewMessage, MapM: Map<T::Message, Out=NewMessage>> DOMNode for
    WithMessageMap<T, NewMessage, MapM>
{
    type Message = NewMessage;
    fn key(&self) -> Option<usize> { self.0.key() }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.0.get_attribute(index)
    }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.0.process_listeners::<MappedListenerProcessor<NewMessage, T::Message, P, MapM>>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.0.process_children::<P>(acc)
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
    fn key(&self) -> Option<usize> { self.node.key() }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        let attributes = self.attributes.as_ref();
        attributes
            .get(index)
            .or_else(|| self.node.get_attribute(index - attributes.len()))
    }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.node.process_listeners::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.node.process_children::<P>(acc)
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { self.node.value() }
}

/// Wrapper for `DOMNode`s that adds listeners.
pub struct WithListeners<T: DOMNode, L: Listeners> {
    node: T,
    listeners: L,
}

impl<T, L> DOMNode for WithListeners<T, L> where T: DOMNode, L: Listeners<Message=T::Message> {
    type Message = T::Message;
    fn key(&self) -> Option<usize> { self.node.key() }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.node.get_attribute(index)
    }
    fn process_listeners<P: ListenerProcessor<Self::Message>>
        (&self, acc: &mut P::Acc) -> Result<(), P::Error>
    {
        self.listeners.process_all::<P>(acc)?;
        self.node.process_listeners::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.node.process_children::<P>(acc)
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
    fn key(&self) -> Option<usize> { (*self).key() }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        (*self).get_attribute(index)
    }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        (*self).process_listeners::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        (*self).process_children::<P>(acc)
    }
    fn value<'b>(&'b self) -> DOMValue<'b> { (*self).value() }
}

pub trait IntoNode<M> {
    type Node: DOMNode<Message = M>;
    fn into_node(self) -> Self::Node;
}

#[cfg(any(feature = "use_std", test))]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StringNode<Message>(String, PhantomData<Message>);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StringRefNode<Message>(&'static str, PhantomData<Message>);

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
    fn key(&self) -> Option<usize> { None }
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn process_children<P: DOMNodeProcessor>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
        Ok(())
    }
    fn value<'a>(&'a self) -> DOMValue<'a> { DOMValue::Text(&self.0) }
}

impl<M> DOMNode for StringRefNode<M> {
    type Message = M;
    fn key(&self) -> Option<usize> { None }
    fn get_attribute(&self, _index: usize) -> Option<&KeyValue> { None }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, _acc: &mut P::Acc) -> Result<(), P::Error> {
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
