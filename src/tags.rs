/// Tags, such as `div` or `table`.
///
/// To create a `Tag` `DOMNode`, simply import the tag function
/// and call it with a type that implements `Into<TagProperties>`.
///
/// Example:
///
/// TODO

use {DOMNode, DOMNodes, DOMValue, KeyValue, Listeners};
use empty::{empty, EmptyNodes, empty_listeners, EmptyListeners};

pub struct TagProperties<
    Children: DOMNodes,
    Attributes: AsRef<[KeyValue]>,
    Listens: Listeners<Message=Children::Message>>
{
    children: Children,
    key: Option<u32>,
    attributes: Attributes,
    listeners: Listens,
}

type EmptyAttrs = [KeyValue; 0];

pub fn attributes<A: AsRef<[KeyValue]>>(attrs: A) -> Attrs<A> {
    Attrs(attrs)
}

pub struct Attrs<A: AsRef<[KeyValue]>>(A);

// No children, attributes, or listeners
impl<M> From<()> for TagProperties<EmptyNodes<M>, EmptyAttrs, EmptyListeners<M>> {
    fn from(_props: ()) -> TagProperties<EmptyNodes<M>, EmptyAttrs, EmptyListeners<M>> {
        TagProperties {
            children: empty(),
            key: None,
            attributes: [],
            listeners: empty_listeners(),
        }
    }
}

// Just children
impl<C: DOMNodes> From<C> for TagProperties<C, EmptyAttrs, EmptyListeners<C::Message>> {
    fn from(nodes: C) -> TagProperties<C, EmptyAttrs, EmptyListeners<C::Message>> {
        TagProperties {
            children: nodes,
            key: None,
            attributes: [],
            listeners: empty_listeners(),
        }
    }
}

// Just attributes
impl<M, A: AsRef<[KeyValue]>>
    From<Attrs<A>> for TagProperties<EmptyNodes<M>, A, EmptyListeners<M>>
{
    fn from(props: Attrs<A>) -> TagProperties<EmptyNodes<M>, A, EmptyListeners<M>> {
        TagProperties {
            children: empty(),
            key: None,
            attributes: props.0,
            listeners: empty_listeners(),
        }
    }
}

// Just listeners
impl<L: Listeners> From<L> for TagProperties<EmptyNodes<L::Message>, EmptyAttrs, L>
{
    fn from(props: L) -> TagProperties<EmptyNodes<L::Message>, EmptyAttrs, L> {
        TagProperties {
            children: empty(),
            key: None,
            attributes: [],
            listeners: props,
        }
    }
}

// (attributes, children)
impl<C: DOMNodes, A: AsRef<[KeyValue]>>
    From<(Attrs<A>, C)> for TagProperties<C, A, EmptyListeners<C::Message>>
{
    fn from(props: (Attrs<A>, C)) -> TagProperties<C, A, EmptyListeners<C::Message>> {
        TagProperties {
            children: props.1,
            key: None,
            attributes: (props.0).0,
            listeners: empty_listeners(),
        }
    }
}

// (attributes, listeners)
impl<A: AsRef<[KeyValue]>, L: Listeners>
    From<(Attrs<A>, L)> for TagProperties<EmptyNodes<L::Message>, A, L>
{
    fn from(props: (Attrs<A>, L)) -> TagProperties<EmptyNodes<L::Message>, A, L> {
        TagProperties {
            children: empty(),
            key: None,
            attributes: (props.0).0,
            listeners: props.1,
        }
    }
}

// (listeners, attributes)
impl<A: AsRef<[KeyValue]>, L: Listeners>
    From<(L, Attrs<A>)> for TagProperties<EmptyNodes<L::Message>, A, L>
{
    fn from(props: (L, Attrs<A>)) -> TagProperties<EmptyNodes<L::Message>, A, L> {
        TagProperties {
            children: empty(),
            key: None,
            attributes: (props.1).0,
            listeners: props.0,
        }
    }
}

// (listeners, children)
impl<C: DOMNodes, L: Listeners<Message=<C as DOMNodes>::Message>>
    From<(L, C)> for TagProperties<C, EmptyAttrs, L>
{
    fn from(props: (L, C)) -> TagProperties<C, EmptyAttrs, L> {
        TagProperties {
            children: props.1,
            key: None,
            attributes: [],
            listeners: props.0,
        }
    }
}

// (attributes, listeners, children)
impl<C: DOMNodes, A: AsRef<[KeyValue]>, L: Listeners<Message=<C as DOMNodes>::Message>>
    From<(Attrs<A>, L, C)> for TagProperties<C, A, L>
{
    fn from(props: (Attrs<A>, L, C)) -> TagProperties<C, A, L> {
        TagProperties {
            children: props.2,
            key: None,
            attributes: (props.0).0,
            listeners: props.1,
        }
    }
}

// (listeners, attributes, children)
impl<C: DOMNodes, A: AsRef<[KeyValue]>, L: Listeners<Message=<C as DOMNodes>::Message>>
    From<(L, Attrs<A>, C)> for TagProperties<C, A, L>
{
    fn from(props: (L, Attrs<A>, C)) -> TagProperties<C, A, L> {
        TagProperties {
            children: props.2,
            key: None,
            attributes: (props.1).0,
            listeners: props.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Tag<
    Children: DOMNodes,
    Attributes: AsRef<[KeyValue]>,
    L: Listeners<Message=Children::Message>>
{
    tagname: &'static str,
    children: Children,
    key: Option<u32>,
    attributes: Attributes,
    listeners: L,
}

impl<C: DOMNodes, A: AsRef<[KeyValue]>, L: Listeners<Message=C::Message>> DOMNode for Tag<C, A, L> {
    type Message = C::Message;
    type Children = C;
    type Listeners = L;
    type WithoutListeners = Tag<C, A, EmptyListeners<Self::Message>>;
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
        let Tag { tagname, children, key, attributes, listeners } = self;
        (
            Tag {
                tagname: tagname,
                children: children,
                key: key,
                attributes: attributes,
                listeners: empty_listeners()
            },
            listeners
        )
    }
    fn value(&self) -> DOMValue {
        DOMValue::Element {
            tag: self.tagname,
        }
    }
}

#[cfg(feature = "use_std")]
use std::fmt;
#[cfg(feature = "use_std")]
impl<C, A, L> fmt::Display for Tag<C, A, L>
    where
    C: DOMNodes,
    A: AsRef<[KeyValue]>,
    L: Listeners<Message=C::Message>
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.displayable().fmt(formatter)
    }
}


macro_rules! impl_tags {
    ($($tagname:ident),*) => { $(
        /// Create a tag of the given type
        ///
        ///
        pub fn $tagname<
            C: DOMNodes,
            A: AsRef<[KeyValue]>,
            L: Listeners<Message=C::Message>,
            T: Into<TagProperties<C, A, L>>
            >(properties: T)
            -> Tag<C, A, L>
        {
            let TagProperties {
                children,
                key,
                attributes,
                listeners,
            } = properties.into();

            Tag {
                tagname: stringify!($tagname),
                children: children,
                key: key,
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
