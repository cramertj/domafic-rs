extern crate libc;
use {DOMValue, DOMNode, DOMNodes, Listener, Listeners};
use keys::{Keys, KeyIter};
use events::Event;
use processors::{DOMNodeProcessor, ListenerProcessor};
use std::marker::PhantomData;

pub trait Updater<State, Message> {
    fn update(&self, &mut State, Message, KeyIter) -> ();
}
impl<F, S, M> Updater<S, M> for F where F: Fn(&mut S, M, KeyIter) -> () {
    fn update(&self, state: &mut S, msg: M, keys: KeyIter) -> () {
        (self)(state, msg, keys)
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
        DOMNode, DOMNodes, Event, Listener, Updater, Keys,
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
                __domafic_pool_free=[];\
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
                        var index = __domafic_pool_free.pop();\
                        if (index) { __domafic_pool[index] = elem; return index; }\
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
                        var index = __domafic_pool_free.pop();\
                        if (index) { __domafic_pool[index] = elem; return index; }\
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
                        var index = __domafic_pool_free.pop();\
                        if (index) { __domafic_pool[index] = elem; return index; }\
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

    unsafe extern fn handle_listener<D, U, R, S>(
        listener_data_c_ptr: *const libc::c_void,
        listener_vtable_c_ptr: *const libc::c_void,
        system_c_ptr: *mut libc::c_void,
        root_node_id: libc::c_int,
        keys_size: libc::c_uint,
        key_1: libc::c_uint,
        key_2: libc::c_uint,
        key_3: libc::c_uint,
        key_4: libc::c_uint,
        key_5: libc::c_uint,
        key_6: libc::c_uint,
        key_7: libc::c_uint,
        key_8: libc::c_uint,
        key_9: libc::c_uint,
        key_10: libc::c_uint,
        key_11: libc::c_uint,
        key_12: libc::c_uint,
        key_13: libc::c_uint,
        key_14: libc::c_uint,
        key_15: libc::c_uint,
        key_16: libc::c_uint,
        key_17: libc::c_uint,
        key_18: libc::c_uint,
        key_19: libc::c_uint,
        key_20: libc::c_uint,
        key_21: libc::c_uint,
        key_22: libc::c_uint,
        key_23: libc::c_uint,
        key_24: libc::c_uint,
        key_25: libc::c_uint,
        key_26: libc::c_uint,
        key_27: libc::c_uint,
        key_28: libc::c_uint,
        key_29: libc::c_uint,
        key_30: libc::c_uint,
        key_31: libc::c_uint,
        key_32: libc::c_uint,
    )
        where
        (D, U, R, S): Sized,
        D: DOMNode,
        D::Message: 'static,
        U: Updater<S, D::Message>,
        R: Renderer<S, Rendered=D>
    {
        let listener_ref: &mut Listener<Message=D::Message> =
            mem::transmute((listener_data_c_ptr, listener_vtable_c_ptr));
        let system_ptr: *mut (D, U, R, S) = mem::transmute(system_c_ptr);
        let system_ref: &mut (D, U, R, S) = system_ptr.as_mut().unwrap();
        let root_node_element = Element(root_node_id);
        let keys = Keys {
            size: keys_size,
            stack: [
                key_1,
                key_2,
                key_3,
                key_4,
                key_5,
                key_6,
                key_7,
                key_8,
                key_9,
                key_10,
                key_11,
                key_12,
                key_13,
                key_14,
                key_15,
                key_16,
                key_17,
                key_18,
                key_19,
                key_20,
                key_21,
                key_22,
                key_23,
                key_24,
                key_25,
                key_26,
                key_27,
                key_28,
                key_29,
                key_30,
                key_31,
                key_32,
            ]
        };

        let message = listener_ref.handle_event(Event {});

        let (ref mut rendered, ref mut updater, ref mut renderer, ref mut state) = *system_ref;

        // Update state
        updater.update(state, message, keys.into_iter());

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
                keys: Keys::new(),
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
        pub unsafe fn on<D, U, R, S>(
            &self,
            event_name: &str,
            listener_ptr: *const Listener<Message=D::Message>,
            system_ptr: *mut (D, U, R, S),
            root_node_id: libc::c_int,
            keys: Keys,
        )
            where
            (D, U, R, S): Sized, // Make sure *mut (D, U, R, S) is a thin ptr
            D: DOMNode,
            D::Message: 'static,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>
        {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0].addEventListener(\
                        UTF8ToString($1),\
                        function(event) {\
                            Runtime.dynCall('viiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii', $2, [$3, $4, $5,\
                            $6, $7,\
                            $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39\
                            ]);\
                        },\
                        false\
                    );\
                \0";

                let event_name_cstring = CString::new(event_name).unwrap();
                let Keys { size: k_size, stack: k } = keys;
                let (listener_data_c_ptr, listener_vtable_c_ptr):
                    (*const libc::c_void, *const libc::c_void) =
                    mem::transmute(listener_ptr);

                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    event_name_cstring.as_ptr() as libc::c_int,
                    handle_listener::<D, U, R, S> as *const libc::c_void,
                    listener_data_c_ptr,
                    listener_vtable_c_ptr,
                    system_ptr as *const libc::c_void,
                    root_node_id,
                    k_size,
                    k[0],
                    k[1],
                    k[2],
                    k[3],
                    k[4],
                    k[5],
                    k[6],
                    k[7],
                    k[8],
                    k[9],
                    k[10],
                    k[11],
                    k[12],
                    k[13],
                    k[14],
                    k[15],
                    k[16],
                    k[17],
                    k[18],
                    k[19],
                    k[20],
                    k[21],
                    k[22],
                    k[23],
                    k[24],
                    k[25],
                    k[26],
                    k[27],
                    k[28],
                    k[29],
                    k[30],
                    k[31]
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
                const JS: &'static [u8] = b"\
                    delete __domafic_pool[$0];\
                    __domafic_pool_free.push($0);\
                \0";
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
    D::Message: 'static,
    U: Updater<S, D::Message>,
    R: Renderer<S, Rendered=D>
{
    unsafe {
        // Get initial DOMNode
        let rendered = renderer.render(&initial_state);

        // Lives forever on the stack, referenced and mutated in callbacks
        let mut app_system = (rendered, updater, renderer, initial_state);
        let app_system_mut_ptr = (&mut app_system) as *mut (D, U, R, S);

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
            keys: Keys::new(),
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
    keys: Keys,
    document: WebDoc,
    root_node_id: WebId,
    parent_node: &'n WebElement,
}

impl<'a, 'n, D, U, R, S> DOMNodeProcessor<'a, D::Message> for WebWriter<'a, 'n, D, U, R, S>
    where
    D: DOMNode,
    D::Message: 'static,
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
            D::Message: 'static,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>
        {
            let node_key = node.key();
            match node.value() {
                DOMValue::Element { tag: tagname } => {
                    let html_node = acc.document.create_element(tagname).unwrap();
                    for attr in node.attributes() {
                        html_node.set_attribute(attr.0, attr.1);
                    }

                    let keys = if let Some(new_key) = node_key {
                        acc.keys.push(new_key)
                    } else {
                        acc.keys
                    };

                    // Reborrow of *document needed to match lifetimes for 'a
                    let mut new_acc = WebWriterAcc {
                        system_ptr: acc.system_ptr,
                        document: acc.document,
                        root_node_id: acc.root_node_id,
                        parent_node: &html_node,
                        keys: keys,
                    };
                    node.children().process_all::<WebWriter<D, U, R, S>>(&mut new_acc)?;

                    let mut listener_acc = WebListenerWriterAcc {
                        system_ptr: acc.system_ptr,
                        root_node_id: acc.root_node_id,
                        node: &html_node,
                        keys: keys,
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
    node: &'n WebElement,
    keys: Keys,
}

impl<'a, 'n, D, U, R, S> ListenerProcessor<'a, D::Message> for
    WebListenerWriter<'n, D, U, R, S>
    where
    D: DOMNode,
    D::Message: 'static,
    U: Updater<S, D::Message>,
    R: Renderer<S, Rendered=D>,
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
            D::Message: 'static,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>,
        {
            let WebListenerWriterAcc {
                ref system_ptr,
                ref root_node_id,
                ref node,
                ref keys,
            } = *acc;

            unsafe {
                // Transmute to assert that we don't care about lifetimes
                let listener_ptr: *const Listener<Message=D::Message> =
                    ::std::mem::transmute(listener as &'a Listener<Message=D::Message>);
                node.on("click", listener_ptr, *system_ptr, *root_node_id, *keys);
            }

            Ok(())
        }
        add_listener
    }
}
