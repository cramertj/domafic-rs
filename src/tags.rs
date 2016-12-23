use {DOMNode, DOMNodes, DOMValue, KeyValue, Listeners};
use processors::{DOMNodeProcessor, ListenerProcessor};
use empty::{empty_listeners, EmptyListeners};

#[cfg(not(any(feature = "use_std", test)))]
extern crate core as std;
use std::marker::PhantomData;

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
        TagProperties {
            children: props.1,
            key: None,
            attributes: (props.0).0,
            listeners: empty_listeners(),
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
