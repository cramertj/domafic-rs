/// Tags, such as `div` or `table`.
///
/// To create a `Tag` `DomNode`, simply import the tag function
/// and call it with a type that implements `Into<TagProperties>`.
///
/// Example:
///
/// TODO

use {DomNode, DomNodes, DomValue, KeyValue, Listeners};
use processors::{DomNodeProcessor, EmptyListeners};

use opt_std::marker::PhantomData;

/// Properties used to create a `Tag` `DomNode`.
///
/// This is primarily used as an input (via `Into<TagProperties>`) for the various tag functions.
/// Note the large number of `From/Into` impls for this struct. Thes allows users to avoid fully
/// specifying all the fields for `TagProperties` by simply calling the tag function with the
/// appropriate combination of listeners, attributes, and children.
///
/// Note that multiple listeners or multiple children must be grouped into a single tuple.
pub struct TagProperties<
    Message,
    Children: DomNodes<Message>,
    Attributes: AsRef<[KeyValue]>,
    Listens: Listeners<Message>>
{
    children: Children,
    key: Option<u32>,
    attributes: Attributes,
    listeners: Listens,
    msg_marker: PhantomData<Message>,
}

type EmptyAttrs = [KeyValue; 0];

/// Create an attributes (`Attrs`) struct from the given array of key-value pairs.
///
/// Use this function to create a tag with a given list of attributes.
///
/// Example:
///
/// ```rust
/// use domafic::DomNode;
/// use domafic::tags::{attributes, div};
/// use domafic::AttributeValue::Str;
/// use std::marker::PhantomData;
///
/// let div_with_attrs = div((
///     attributes([("key", Str("value"))]),
///     // We need to manually mark the message type since it can't be inferred
///     PhantomData::<()>
/// ));
/// assert_eq!(div_with_attrs.get_attribute(0), Some(&("key", Str("value"))));
/// ```
pub fn attributes<A: AsRef<[KeyValue]>>(attrs: A) -> Attrs<A> {
    Attrs(attrs)
}

/// Wrapper for an array of attributes re
pub struct Attrs<A: AsRef<[KeyValue]>>(A);

// Just children
impl<M, C: DomNodes<M>> From<C> for TagProperties<M, C, EmptyAttrs, EmptyListeners> {
    fn from(nodes: C) -> TagProperties<M, C, EmptyAttrs, EmptyListeners> {
        TagProperties {
            children: nodes,
            key: None,
            attributes: [],
            listeners: EmptyListeners,
            msg_marker: PhantomData,
        }
    }
}

// Just attributes
impl<M, A: AsRef<[KeyValue]>>
    From<Attrs<A>> for TagProperties<M, (), A, EmptyListeners>
{
    fn from(props: Attrs<A>) -> TagProperties<M, (), A, EmptyListeners> {
        TagProperties {
            children: (),
            key: None,
            attributes: props.0,
            listeners: EmptyListeners,
            msg_marker: PhantomData,
        }
    }
}

// Just Listeners
impl<M, L: Listeners<M>>
    From<L> for TagProperties<M, (), EmptyAttrs, L>
{
    fn from(props: L) -> TagProperties<M, (), EmptyAttrs, L> {
        TagProperties {
            children: (),
            key: None,
            attributes: [],
            listeners: props,
            msg_marker: PhantomData,
        }
    }
}

// (attributes, children)
impl<M, C: DomNodes<M>, A: AsRef<[KeyValue]>>
    From<(Attrs<A>, C)> for TagProperties<M, C, A, EmptyListeners>
{
    fn from(props: (Attrs<A>, C)) -> TagProperties<M, C, A, EmptyListeners> {
        TagProperties {
            children: props.1,
            key: None,
            attributes: (props.0).0,
            listeners: EmptyListeners,
            msg_marker: PhantomData,
        }
    }
}

// (attributes, listeners)
impl<M, A: AsRef<[KeyValue]>, L: Listeners<M>>
    From<(Attrs<A>, L)> for TagProperties<M, (), A, L>
{
    fn from(props: (Attrs<A>, L)) -> TagProperties<M, (), A, L> {
        TagProperties {
            children: (),
            key: None,
            attributes: (props.0).0,
            listeners: props.1,
            msg_marker: PhantomData,
        }
    }
}

// (listeners, children)
impl<M, C: DomNodes<M>, L: Listeners<M>>
    From<(L, C)> for TagProperties<M, C, EmptyAttrs, L>
{
    fn from(props: (L, C)) -> TagProperties<M, C, EmptyAttrs, L> {
        TagProperties {
            children: props.1,
            key: None,
            attributes: [],
            listeners: props.0,
            msg_marker: PhantomData,
        }
    }
}

// (attributes, listeners, children)
impl<M, C: DomNodes<M>, A: AsRef<[KeyValue]>, L: Listeners<M>>
    From<(Attrs<A>, L, C)> for TagProperties<M, C, A, L>
{
    fn from(props: (Attrs<A>, L, C)) -> TagProperties<M, C, A, L> {
        TagProperties {
            children: props.2,
            key: None,
            attributes: (props.0).0,
            listeners: props.1,
            msg_marker: PhantomData,
        }
    }
}

/// A tag element, such as `div` or `span`.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Tag<
    Message,
    Children: DomNodes<Message>,
    Attributes: AsRef<[KeyValue]>,
    L: Listeners<Message>>
{
    tagname: &'static str,
    children: Children,
    key: Option<u32>,
    attributes: Attributes,
    listeners: L,
    msg_marker: PhantomData<Message>,
}

impl<
    M,
    C: DomNodes<M>,
    A: AsRef<[KeyValue]>,
    L: Listeners<M>> DomNodes<M> for Tag<M, C, A, L>
{
    fn process_all<'a, P: DomNodeProcessor<'a, M>>(&'a self, acc: &mut P::Acc) -> Result<(), P::Error> {
        P::get_processor()(acc, self)
    }
}
impl<M, C: DomNodes<M>, A: AsRef<[KeyValue]>, L: Listeners<M>> DomNode<M> for Tag<M, C, A, L> {
    type Children = C;
    type Listeners = L;
    type WithoutListeners = Tag<M, C, A, EmptyListeners>;
    fn key(&self) -> Option<u32> { self.key }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.attributes.as_ref().get(index)
    }
    fn children(&self) -> &Self::Children {
        &self.children
    }
    fn listeners(&self) -> &Self::Listeners {
        &self.listeners
    }
    fn children_and_listeners(&self) -> (&Self::Children, &Self::Listeners) {
        (&self.children, &self.listeners)
    }
    fn split_listeners(self) -> (Self::WithoutListeners, Self::Listeners) {
        let Tag { tagname, children, key, attributes, listeners, msg_marker } = self;
        (
            Tag {
                tagname: tagname,
                children: children,
                key: key,
                attributes: attributes,
                listeners: EmptyListeners,
                msg_marker: msg_marker,
            },
            listeners
        )
    }
    fn value(&self) -> DomValue {
        DomValue::Element {
            tag: self.tagname,
        }
    }
}

#[cfg(any(feature = "use_std", test))]
use std::fmt;
#[cfg(any(feature = "use_std", test))]
impl<M, C, A, L> fmt::Display for Tag<M, C, A, L>
    where
    C: DomNodes<M>,
    A: AsRef<[KeyValue]>,
    L: Listeners<M>
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.displayable().fmt(formatter)
    }
}


macro_rules! impl_tags {
    ($($tagname:ident),*) => { $(
        /// Creates a tag of the given type.
        ///
        /// Note the use of `Into<TagProperties>`. This allows for a wide variety of input
        /// parameters such as `div(())`, `div(...children...)`,
        /// `div((...attributes..., ...children..))`, `div((...attributes..., ...listeners...))`
        /// and more.
        pub fn $tagname<
            M,
            C: DomNodes<M>,
            A: AsRef<[KeyValue]>,
            L: Listeners<M>,
            T: Into<TagProperties<M, C, A, L>>
            >(properties: T)
            -> Tag<M, C, A, L>
        {
            let TagProperties {
                children,
                key,
                attributes,
                listeners,
                msg_marker,
            } = properties.into();

            Tag {
                tagname: stringify!($tagname),
                children: children,
                key: key,
                attributes: attributes,
                listeners: listeners,
                msg_marker: msg_marker,
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
