/// Tags, such as `div` or `table`.
///
/// To create a `Tag` DOMNode, simply import the tag function
/// and call it with a type that implements `Into<TagProperties>`.
///
/// Example:
///
/// TODO

use {DOMNode, DOMNodes, DOMValue, KeyValue, Listeners};
use processors::{DOMNodeProcessor, ListenerProcessor};
use empty::{empty_listeners, EmptyListeners};

pub struct TagProperties<
    Children: DOMNodes,
    Attributes: AsRef<[KeyValue]>,
    Listens: Listeners<Message=Children::Message>>
{
    children: Children,
    key: Option<usize>,
    attributes: Attributes,
    listeners: Listens,
}

type EmptyAttrs = [KeyValue; 0];

pub fn attributes<A: AsRef<[KeyValue]>>(attrs: A) -> Attrs<A> {
    Attrs(attrs)
}

pub struct Attrs<A: AsRef<[KeyValue]>>(A);

// just children
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

pub struct Tag<
    Children: DOMNodes,
    Attributes: AsRef<[KeyValue]>,
    L: Listeners<Message=Children::Message>>
{
    tagname: &'static str,
    children: Children,
    key: Option<usize>,
    attributes: Attributes,
    listeners: L,
}

impl<C: DOMNodes, A: AsRef<[KeyValue]>, L: Listeners<Message=C::Message>> DOMNode for Tag<C, A, L> {
    type Message = C::Message;
    fn key(&self) -> Option<usize> { self.key }
    fn get_attribute(&self, index: usize) -> Option<&KeyValue> {
        self.attributes.as_ref().get(index)
    }
    fn process_listeners<P: ListenerProcessor<Self::Message>>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.listeners.process_all::<P>(acc)
    }
    fn process_children<P: DOMNodeProcessor>(&self, acc: &mut P::Acc) -> Result<(), P::Error> {
        self.children.process_all::<P>(acc)
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
