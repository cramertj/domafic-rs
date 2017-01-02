extern crate libc;
use {DOMValue, DOMNode, DOMNodes, Listener, Listeners};
use events::Event;
use processors::{DOMNodeProcessor, ListenerProcessor};
use std::marker::PhantomData;

pub trait Updater<State, Message> {
    fn update(&self, &mut State, Message) -> ();
}
impl<F, S, M> Updater<S, M> for F where F: Fn(&mut S, M) -> () {
    fn update(&self, state: &mut S, msg: M) -> () {
        (self)(state, msg)
    }
}

pub trait Renderer<State> {
    type Rendered: DOMNode;
    fn render(&self, &State) -> Self::Rendered;
}
impl<F, S, R> Renderer<S> for F where F: Fn(&S) -> R, R: DOMNode {
    type Rendered = R;
    fn render(&self, state: &S) -> Self::Rendered {
        (self)(state)
    }
}

use self::web_interface::{Document as WebDoc, Element as WebElement, JsElementId as WebId};
mod web_interface {
    use super::{
        DOMNode, DOMNodes, Event, Listener, Updater,
        Renderer, WebWriter, WebWriterAcc
    };
    use super::libc;
    use ::std::ffi::CString;
    use ::std::mem;

    extern "C" {
        fn emscripten_asm_const_int(s: *const libc::c_char, ...) -> libc::c_int;
        fn emscripten_pause_main_loop();
        fn emscripten_set_main_loop(m: extern fn(), fps: libc::c_int, infinite: libc::c_int);
    }

    pub type JsElementId = libc::c_int;

    #[derive(Debug)]
    pub struct Element(JsElementId);

    #[derive(Debug, Copy, Clone)]
    pub struct Document(()); // Contains private () so that it can't be created externally

    pub fn init() -> Document {
        const JS: &'static [u8] = b"\
            if('undefined'===typeof __domafic_pool){\
                console.log('Intializing __domafic_pool');\
                __domafic_pool=[];\
            }\
        \0";

        unsafe {
            emscripten_asm_const_int(&JS[0] as *const _ as *const libc::c_char);
        }

        Document(())
    }

    extern fn pause_main_loop() {
        unsafe { emscripten_pause_main_loop(); }
    }

    pub fn main_loop() -> ! {
        unsafe { emscripten_set_main_loop(pause_main_loop, 0, 1); }
        panic!("Emscripten main loop should never return")
    }

    impl Document {
        pub fn element_from_selector(&self, selector: &str) -> Option<Element> {
            let id = {
                unsafe {
                    const JS: &'static [u8] = b"\
                        var elem = document.querySelector(UTF8ToString($0));\
                        if (!elem) {return -1;}\
                        return __domafic_pool.push(elem) - 1;\
                    \0";
                    let selector_cstring = CString::new(selector).unwrap();
                    emscripten_asm_const_int(
                        &JS[0] as *const _ as *const libc::c_char,
                        selector_cstring.as_ptr() as libc::c_int
                    )
                }
            };
            if id < 0 { None } else { Some(Element(id)) }
        }

        pub fn create_element(&self, tagname: &str) -> Option<Element> {
            let id = {
                unsafe {
                    const JS: &'static [u8] = b"\
                        var elem = document.createElement(UTF8ToString($0));\
                        if (!elem) {return -1;}\
                        return __domafic_pool.push(elem) - 1;\
                    \0";
                    let tagname_cstring = CString::new(tagname).unwrap();
                    emscripten_asm_const_int(
                        &JS[0] as *const _ as *const libc::c_char,
                        tagname_cstring.as_ptr() as libc::c_int
                    )
                }
            };
            if id < 0 { None } else { Some(Element(id)) }
        }

        pub fn create_text_node(&self, text: &str) -> Option<Element> {
            let id = {
                unsafe {
                    const JS: &'static [u8] = b"\
                        var elem = document.createTextNode(UTF8ToString($0));\
                        if (!elem) {return -1;}\
                        return __domafic_pool.push(elem) - 1;\
                    \0";
                    let text_cstring = CString::new(text).unwrap();
                    emscripten_asm_const_int(
                        &JS[0] as *const _ as *const libc::c_char,
                        text_cstring.as_ptr() as libc::c_int
                    )
                }
            };
            if id < 0 { None } else { Some(Element(id)) }
        }
    }

    unsafe extern fn handle_listener<L, D, U, R, S>(
        listener_c_ptr: *const libc::c_void,
        system_c_ptr: *mut libc::c_void,
        root_node_id: libc::c_int
    )
        where
        L: Listener<Message=D::Message> + Sized,
        (D, U, R, S): Sized,
        D: DOMNode,
        U: Updater<S, D::Message>,
        R: Renderer<S, Rendered=D>
    {
        let listener_ref: &mut L = mem::transmute(listener_c_ptr);
        let system_ptr: *mut (D, U, R, S) = mem::transmute(system_c_ptr);
        let system_ref: &mut (D, U, R, S) = system_ptr.as_mut().unwrap();
        let root_node_element = Element(root_node_id);

        let message = listener_ref.handle_event(Event {});

        let (ref mut rendered, ref mut updater, ref mut renderer, ref mut state) = *system_ref;

        // Update state
        updater.update(state, message);

        // Render new DOMNode
        *rendered = renderer.render(state);

        // Write new DOMNode to root element
        root_node_element.remove_all_children();
        {
            let mut input = WebWriterAcc {
                system_ptr: system_ptr,
                document: Document(()),
                root_node_id: root_node_id,
                parent_node: &root_node_element,
            };
            rendered.process_all::<WebWriter<D, U, R, S>>(&mut input).unwrap();
        }

        // Don't drop the root node reference
        mem::forget(root_node_element);
    }

    impl Element {
        pub fn get_id(&self) -> JsElementId { self.0 }

        pub fn append(&self, child: &Element) {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0].appendChild(__domafic_pool[$1]);\
                \0";

                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    child.0
                );
            }
        }

        /// Requires that `listener_ptr` and `system_ptr` are valid and that
        /// `root_node_id` is a valid `Element` id throughout the duration of
        /// time that it is possible for this callback to be triggered.
        pub unsafe fn on<L, D, U, R, S>(
            &self,
            event_name: &str,
            listener_ptr: *const L,
            system_ptr: *mut (D, U, R, S),
            root_node_id: libc::c_int,
        )
            where
            L: Listener<Message=D::Message> + Sized,
            (D, U, R, S): Sized,
            D: DOMNode,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>
        {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0].addEventListener(\
                        UTF8ToString($1),\
                        function(event) {\
                            Runtime.dynCall('viii', $2, [$3, $4, $5]);\
                        },\
                        false\
                    );\
                \0";

                let event_name_cstring = CString::new(event_name).unwrap();
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    event_name_cstring.as_ptr() as libc::c_int,
                    handle_listener::<L, D, U, R, S> as *const libc::c_void,
                    listener_ptr as *const libc::c_void,
                    system_ptr as *const libc::c_void,
                    root_node_id
                );
            }
        }

        pub fn remove_all_children(&self) {
            unsafe {
                const JS: &'static [u8] = b"\
                    var elem = __domafic_pool[$0];\
                    while (elem.hasChildNodes()) { elem.removeChild(elem.lastChild); }\
                \0";
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                );
            }
        }

        #[allow(dead_code)]
        pub fn remove_self(&self) {
            unsafe {
                const JS: &'static [u8] = b"\
                    var elem = __domafic_pool[$0];\
                    if (elem.parentNode) { elem.parentNode.removeChild(elem); }\
                \0";
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                );
            }
        }

        pub fn set_attribute(&self, key: &str, value: &str) {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0][UTF8ToString($1)] = UTF8ToString($2);\
                \0";
                let key_cstring = CString::new(key).unwrap();
                let value_cstring = CString::new(value).unwrap();
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    key_cstring.as_ptr() as libc::c_int,
                    value_cstring.as_ptr() as libc::c_int
                );
            }
        }
    }

    impl Drop for Element {
        fn drop(&mut self) {
            unsafe {
                const JS: &'static [u8] = b"delete __domafic_pool[$0];\0";
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                );
            }
        }
    }
}

pub fn run<D, U, R, S>(element_selector: &str, updater: U, renderer: R, initial_state: S) -> !
    where
    D: DOMNode,
    U: Updater<S, D::Message>,
    R: Renderer<S, Rendered=D>
{
    unsafe {
        // Get initial DOMNode
        let rendered = renderer.render(&initial_state);

        // Lives forever on the stack, referenced and mutated in callbacks
        let mut app_system = (rendered, updater, renderer, initial_state);
        let app_system_mut_ref = &mut app_system;
        let app_system_mut_ptr = app_system_mut_ref as *mut (D, U, R, S);

        // Initialize the browser system
        let document = web_interface::init();
        let root_node_element =
            document.element_from_selector(element_selector).unwrap();

        // Draw initial DOMNode to browser
        root_node_element.remove_all_children();
        let mut input = WebWriterAcc {
            system_ptr: app_system_mut_ptr,
            document: document,
            root_node_id: root_node_element.get_id(),
            parent_node: &root_node_element,
        };
        (*app_system_mut_ptr).0.process_all::<WebWriter<D, U, R, S>>(&mut input).unwrap();

        web_interface::main_loop()
    }
}

struct WebWriter<'a, 'n, D, U, R, S>(
    PhantomData<(&'a (), &'n (), D, U, R, S)>
);
struct WebWriterAcc<'n, D, U, R, S> {
    system_ptr: *mut (D, U, R, S),
    document: WebDoc,
    root_node_id: WebId,
    parent_node: &'n WebElement,
}

impl<'a, 'n, D, U, R, S> DOMNodeProcessor<'a, D::Message> for WebWriter<'a, 'n, D, U, R, S>
    where
    D: DOMNode,
    U: Updater<S, D::Message>,
    R: Renderer<S, Rendered=D>
{
    type Acc = WebWriterAcc<'n, D, U, R, S>;
    type Error = ();

    fn get_processor<T: DOMNode<Message=D::Message>>() -> fn(&mut Self::Acc, &'a T) -> Result<(), Self::Error> {
        fn add_node<'a, 'n, T, D, U, R, S>(
            acc: &mut WebWriterAcc<'n, D, U, R, S>,
            node: &'a T) -> Result<(), ()>
            where
            T: DOMNode<Message=D::Message>,
            D: DOMNode,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>
        {
            match node.value() {
                DOMValue::Element { tag: tagname } => {
                    let html_node = acc.document.create_element(tagname).unwrap();
                    for attr in node.attributes() {
                        html_node.set_attribute(attr.0, attr.1);
                    }

                    // Reborrow of *document needed to match lifetimes for 'a
                    let mut new_acc = WebWriterAcc {
                        system_ptr: acc.system_ptr,
                        document: acc.document,
                        root_node_id: acc.root_node_id,
                        parent_node: &html_node,
                    };
                    node.children().process_all::<WebWriter<D, U, R, S>>(&mut new_acc)?;

                    let mut listener_acc = WebListenerWriterAcc {
                        system_ptr: acc.system_ptr,
                        root_node_id: acc.root_node_id,
                        node: &html_node,
                    };
                    node.listeners().process_all::<WebListenerWriter<D, U, R, S>>(
                        &mut listener_acc
                    )?;

                    acc.parent_node.append(&html_node);
                }
                DOMValue::Text(text) => {
                    let text_element = acc.document.create_text_node(text).unwrap();
                    acc.parent_node.append(&text_element);
                }
            }
            Ok(())
        }
        add_node
    }
}

struct WebListenerWriter<
    'n,
    D: DOMNode,
    U: Updater<S, D::Message>,
    R: Renderer<S, Rendered=D>,
    S>
(
    PhantomData<(&'n (), D, U, R, S)>
);

struct WebListenerWriterAcc<'n, D, U, R, S> {
    system_ptr: *mut (D, U, R, S),
    root_node_id: WebId,
    node: &'n WebElement
}

impl<'a, 'n, D, U, R, S> ListenerProcessor<'a, D::Message> for
    WebListenerWriter<'n, D, U, R, S>
    where
    D: DOMNode,
    U: Updater<S, D::Message>,
    R: Renderer<S, Rendered=D>
{
    type Acc = WebListenerWriterAcc<'n, D, U, R, S>;
    type Error = ();

    fn get_processor<L: Listener<Message=D::Message>>() -> fn(&mut Self::Acc, &'a L) -> Result<(), Self::Error> {
        fn add_listener<'a, 'n, D, U, R, S, L> (
            acc: &mut WebListenerWriterAcc<'n, D, U, R, S>,
            listener: &'a L) -> Result<(), ()>
            where
            L: Listener<Message=D::Message>,
            D: DOMNode,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>
        {
            let WebListenerWriterAcc {
                ref system_ptr,
                ref root_node_id,
                ref node,
            } = *acc;

            unsafe {
                node.on("click", listener as *const L, *system_ptr, *root_node_id);
            }

            Ok(())
        }
        add_listener
    }
}
