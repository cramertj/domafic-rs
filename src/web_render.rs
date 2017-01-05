use DOMNode;
use keys::KeyIter;

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

#[cfg(target_os = "emscripten")]
pub fn run<D, U, R, S>(element_selector: &str, updater: U, renderer: R, initial_state: S) -> !
        where
        D: DOMNode,
        D::Message: 'static,
        U: Updater<S, D::Message>,
        R: Renderer<S, Rendered=D>
{
    private::run(element_selector, updater, renderer, initial_state)
}

#[cfg(not(target_os = "emscripten"))]
pub fn run<D, U, R, S>(element_selector: &str, updater: U, renderer: R, initial_state: S) -> !
        where
        D: DOMNode,
        D::Message: 'static,
        U: Updater<S, D::Message>,
        R: Renderer<S, Rendered=D>
{
    let _ = (element_selector, updater, renderer, initial_state);
    panic!("Target does not support web_render::run (requires emscripten).")
}

#[cfg(target_os = "emscripten")]
mod private {
    extern crate libc;

    use super::{Updater, Renderer};
    use {DOMNode, DOMValue, KeyValue, Listener};
    use events::Event;
    use keys::Keys;
    use processors::{DOMNodes, Listeners, DOMNodeProcessor, ListenerProcessor};

    use std::ffi::CString;
    use std::marker::PhantomData;
    use std::mem;

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

            // Initialize the browser system
            let document = web_init();
            let root_node_element =
                document.element_from_selector(element_selector).unwrap();
            root_node_element.remove_all_children();

            // Lives forever on the stack, referenced and mutated in callbacks
            let mut app_system = (
                rendered,
                updater,
                renderer,
                initial_state,
                VDOMNode {
                    value: VNodeValue::Tag("N/A - root"),
                    keys: Keys::new(),
                    web_element: root_node_element,
                    attributes: Vec::new(),
                    listeners: Vec::new(),
                    children: Vec::new(),
                }
            );
            let app_system_mut_ptr = (&mut app_system) as *mut (D, U, R, S, VDOMNode<D::Message>);

            // Draw initial DOMNode to browser
            let mut node_index = 0;
            let mut input = WebWriterAcc {
                system_ptr: app_system_mut_ptr,
                document: document,
                keys: Keys::new(),
                parent_element: &(*app_system_mut_ptr).4.web_element,
                node_level: &mut (*app_system_mut_ptr).4.children,
                node_index: &mut node_index,
            };
            (*app_system_mut_ptr).0.process_all::<WebWriter<D, U, R, S>>(&mut input).unwrap();

            run_main_web_loop()
        }
    }

    extern "C" {
        fn emscripten_asm_const_int(s: *const libc::c_char, ...) -> libc::c_int;
        fn emscripten_pause_main_loop();
        fn emscripten_set_main_loop(m: extern fn(), fps: libc::c_int, infinite: libc::c_int);
    }

    type JsElementId = libc::c_int;

    #[derive(Debug)]
    struct WebElement(JsElementId);

    #[derive(Debug, Copy, Clone)]
    struct WebDocument(()); // Contains private () so that it can't be created externally

    fn web_init() -> WebDocument {
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

        WebDocument(())
    }

    extern fn pause_main_web_loop() {
        unsafe { emscripten_pause_main_loop(); }
    }

    fn run_main_web_loop() -> ! {
        unsafe { emscripten_set_main_loop(pause_main_web_loop, 0, 1); }
        panic!("Emscripten main loop should never return")
    }

    impl WebDocument {
        fn element_from_selector(&self, selector: &str) -> Option<WebElement> {
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
            if id < 0 { None } else { Some(WebElement(id)) }
        }

        fn create_element(&self, tagname: &str) -> Option<WebElement> {
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
            if id < 0 { None } else { Some(WebElement(id)) }
        }

        fn create_text_node(&self, text: &str) -> Option<WebElement> {
            let id = {
                unsafe {
                    const JS: &'static [u8] = b"\
                        var text = document.createTextNode(UTF8ToString($0));\
                        if (!text) {return -1;}\
                        var elem = document.createElement('span');\
                        elem.appendChild(text);\
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
            if id < 0 { None } else { Some(WebElement(id)) }
        }
    }

    unsafe extern fn handle_listener<D, U, R, S>(
        listener_data_c_ptr: *const libc::c_void,
        listener_vtable_c_ptr: *const libc::c_void,
        system_c_ptr: *mut libc::c_void,
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
        let system_ptr: *mut (D, U, R, S, VDOMNode<D::Message>) = mem::transmute(system_c_ptr);
        let system_ref: &mut (D, U, R, S, VDOMNode<D::Message>) = system_ptr.as_mut().unwrap();
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

        let (
            ref mut rendered,
            ref mut updater,
            ref mut renderer,
            ref mut state,
            ref mut vdom_root,
        ) = *system_ref;

        // Update state
        updater.update(state, message, keys.into_iter());

        // Render new DOMNode
        *rendered = renderer.render(state);

        // Write new DOMNode to root element
        {
            let mut node_index = 0;
            let mut input = WebWriterAcc {
                system_ptr: system_ptr,
                document: WebDocument(()),
                keys: Keys::new(),
                parent_element: &vdom_root.web_element,
                node_level: &mut vdom_root.children,
                node_index: &mut node_index,
            };
            rendered.process_all::<WebWriter<D, U, R, S>>(&mut input).unwrap();
        }
    }

    impl WebElement {

        #[allow(dead_code)]
        fn append(&self, child: &WebElement) {
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

        fn insert(&self, index: usize, child: &WebElement) {
            let err = unsafe {
                const JS: &'static [u8] = b"\
                    var parent = __domafic_pool[$0];\
                    if ($2 > parent.children.length) { return -1; }\
                    if ($2 == parent.children.length) {\
                        parent.appendChild(__domafic_pool[$1]);\
                    } else {\
                        parent.insertBefore(__domafic_pool[$1], parent.children[$2]);\
                    }\
                    return 0;\
                \0";

                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    child.0,
                    index as libc::c_int
                )
            };

            // Must panic on error because failure to properly add/remove nodes
            // containing listeners can cause memory unsafety
            if err < 0 { panic!("Attempted to insert child DOM element out of bounds") }
        }

        fn move_child(&self, old_index: usize, new_index: usize) {
            let err = unsafe {
                const JS: &'static [u8] = b"\
                    var parent = __domafic_pool[$0];\
                    if ($1 > parent.children.length) { return -1; }\
                    if ($2 > parent.children.length) { return -1; }\
                    var element = parent.children[$1];\
                    if ($2 == parent.children.length) {\
                        parent.appendChild(element);\
                    } else {\
                        parent.insertBefore(element, parent.children[$2]);\
                    }\
                    return 0;\
                \0";

                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    old_index as libc::c_int,
                    new_index as libc::c_int
                )
            };

            // Must panic on error because failure to properly add/remove nodes
            // containing listeners can cause memory unsafety
            if err < 0 { panic!("Attempted to move child DOM element out of bounds") }
        }

        /// Requires that `listener_ptr` and `system_ptr` are valid and that
        /// `root_node_id` is a valid `WebElement` id throughout the duration of
        /// time that it is possible for this callback to be triggered.
        /// Returns an element that is a reference to the created function
        unsafe fn set_listener<D, U, R, S>(
            &self,
            event_name: &str,
            listener_ptr: *const Listener<Message=D::Message>,
            system_ptr: *mut (D, U, R, S, VDOMNode<D::Message>),
            keys: Keys,
        ) -> WebElement
            where
            (D, U, R, S): Sized, // Make sure *mut (D, U, R, S) is a thin ptr
            D: DOMNode,
            D::Message: 'static,
            U: Updater<S, D::Message>,
            R: Renderer<S, Rendered=D>
        {
            unsafe {
                const JS: &'static [u8] = b"\
                    var callback = function(event) {\
                        Runtime.dynCall('viiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii', $2, [$3, $4, $5, $6,\
                        $7,\
                        $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38\
                        ]);\
                    };\
                    __domafic_pool[$0].addEventListener(\
                        UTF8ToString($1),\
                        callback,\
                        false\
                    );\
                    var index = __domafic_pool_free.pop();\
                    if (index) { __domafic_pool[index] = callback; return index; }\
                    return __domafic_pool.push(callback) - 1;\
                \0";

                let event_name_cstring = CString::new(event_name).unwrap();
                let Keys { size: k_size, stack: k } = keys;
                let (listener_data_c_ptr, listener_vtable_c_ptr):
                    (*const libc::c_void, *const libc::c_void) =
                    mem::transmute(listener_ptr);

                WebElement(emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    event_name_cstring.as_ptr() as libc::c_int,
                    handle_listener::<D, U, R, S> as *const libc::c_void,
                    listener_data_c_ptr,
                    listener_vtable_c_ptr,
                    system_ptr as *const libc::c_void,
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
                ))
            }
        }

        fn remove_listener(&self, event_name: &str, listener: &WebElement) {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0].removeEventListener(\
                        UTF8ToString($1), __domafic_pool[$2]);\
                \0";
                let event_name_cstring = CString::new(event_name).unwrap();
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    event_name_cstring.as_ptr() as libc::c_int,
                    listener.0,
                );
            }
        }

        fn remove_all_children(&self) {
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
        fn remove_self(&self) {
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

        fn remove_attribute(&self, key: &str) {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0].removeAttribute(UTF8ToString($2));\
                \0";
                let key_cstring = CString::new(key).unwrap();
                emscripten_asm_const_int(
                    &JS[0] as *const _ as *const libc::c_char,
                    self.0,
                    key_cstring.as_ptr() as libc::c_int,
                );
            }
        }

        fn set_attribute(&self, key: &str, value: &str) {
            unsafe {
                const JS: &'static [u8] = b"\
                    __domafic_pool[$0].setAttribute(UTF8ToString($1), UTF8ToString($2));\
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

    impl Drop for WebElement {
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

    #[derive(Debug, Clone, Eq, PartialEq)]
    enum VNodeValue {
        Text(String),
        Tag(&'static str),
    }
    #[derive(Debug)]
    struct VDOMNode<Message: 'static> {
        value: VNodeValue,
        keys: Keys,
        web_element: WebElement,
        attributes: Vec<KeyValue>,
        listeners: Vec<(*const Listener<Message=Message>, WebElement)>,
        children: VDOMLevel<Message>,
    }
    type VDOMLevel<Message: 'static> = Vec<VDOMNode<Message>>;

    struct WebWriter<'a, 'n, D, U, R, S>(
        PhantomData<(&'a (), &'n (), D, U, R, S)>
    );
    struct WebWriterAcc<'n, D: DOMNode, U, R, S> where D::Message: 'static {
        system_ptr: *mut (D, U, R, S, VDOMNode<D::Message>),
        keys: Keys,
        document: WebDocument,
        parent_element: &'n WebElement,
        node_level: &'n mut VDOMLevel<D::Message>,
        node_index: &'n mut usize,
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

                let vnode_value = match node.value() {
                    DOMValue::Element { tag } => VNodeValue::Tag(tag),
                    DOMValue::Text(text) => VNodeValue::Text(text.to_string()),
                };

                let keys = if let Some(new_key) = node.key() {
                    acc.keys.push(new_key)
                } else {
                    acc.keys
                };

                let listeners = {
                    let mut listeners = Vec::new();
                    node.listeners().process_all::<ListenersToVec<D::Message>>(&mut listeners)?;
                    listeners
                };

                let vnode_match_opt_index = {
                    let mut vnode_match_opt_index = None;
                    let mut trial_index = *acc.node_index;
                    while let Some(trial_vnode) = acc.node_level.get(trial_index) {
                        // Match iff "keys" and "value" are equal and the new listeners are a subset
                        // of the old listeners. Cannot match elements with lower indices than
                        // `acc.node_index`, as they are the output of prior calls to `add_node`.
                        if (trial_vnode.keys == keys) &&
                            (trial_vnode.value == vnode_value)
                        {
                            vnode_match_opt_index = Some(trial_index);
                            break;
                        } else {
                            trial_index += 1;
                        }
                    }
                    vnode_match_opt_index
                };

                if let Some(vnode_index) = vnode_match_opt_index {
                    // Modify the existing element
                    // Add new listeners, unify attributes, unify children

                    {
                        let mut vnode = &mut acc.node_level[vnode_index];

                        // Remove excess listeners
                        {
                            let mut i = 0;
                            while i < vnode.listeners.len() {
                                if !listeners.contains(&vnode.listeners[i].0) {
                                    vnode.web_element.remove_listener("click", &vnode.listeners[i].1);
                                    vnode.listeners.remove(i);
                                } else {
                                    i += 1;
                                }
                            }
                        }

                        // Add new listeners
                        for listener in listeners {
                            if !vnode.listeners.iter().map(|x| x.0).any(|x| x == listener) {
                                let element = unsafe {
                                    vnode.web_element.set_listener(
                                        "click", listener, acc.system_ptr, keys)
                                };
                                vnode.listeners.push((listener, element));
                            }
                        }

                        // Remove excess attributes
                        {
                            let mut i = 0;
                            while i < vnode.attributes.len() {
                                let old_attribute = vnode.attributes[i];
                                if !node.attributes().any(|attr| *attr == old_attribute) {
                                    vnode.web_element.remove_attribute(old_attribute.0);
                                    vnode.attributes.remove(i);
                                } else {
                                    i += 1;
                                }
                            }
                        }

                        // Add new attributes
                        for new_attribute in node.attributes() {
                            vnode.web_element.set_attribute(new_attribute.0, new_attribute.1);
                            vnode.attributes.push(*new_attribute);
                        }

                        // To the children!
                        let mut child_node_index = 0;
                        {
                            let mut new_acc = WebWriterAcc {
                                system_ptr: acc.system_ptr,
                                keys: keys,
                                document: acc.document,
                                parent_element: &vnode.web_element,
                                node_level: &mut vnode.children,
                                node_index: &mut child_node_index,
                            };
                            node.children().process_all::<WebWriter<D, U, R, S>>(&mut new_acc)?;
                        }
                        // Remove DOM elements left over from the last render that weren't repurposed
                        while child_node_index < vnode.children.len() {
                            let unused_dom_element = vnode.children.pop().unwrap();
                            unused_dom_element.web_element.remove_self();
                        }
                    }

                    // Move the element if the new index is different from the old one
                    if *acc.node_index != vnode_index {
                        acc.parent_element.move_child(vnode_index, *acc.node_index);
                        let old_vnode = acc.node_level.remove(vnode_index);
                        acc.node_level.insert(*acc.node_index, old_vnode);
                    }
                } else {
                    // Construct as a new element

                    let html_element = match node.value() {
                        DOMValue::Element { tag } => {
                            acc.document.create_element(tag).unwrap()},
                        DOMValue::Text(text) =>
                            acc.document.create_text_node(text).unwrap(),
                    };

                    let mut listeners_and_elements = Vec::new();
                    for listener in listeners {
                        let element = unsafe {
                            html_element.set_listener("click", listener, acc.system_ptr, keys)
                        };
                        listeners_and_elements.push((listener, element));
                    }

                    let mut vnode_attributes = Vec::new();
                    for attr in node.attributes() {
                        html_element.set_attribute(attr.0, attr.1);
                        vnode_attributes.push((attr.0, attr.1));
                    }

                    let mut vnode = VDOMNode {
                        value: vnode_value,
                        keys: keys,
                        web_element: html_element,
                        attributes: vnode_attributes,
                        listeners: listeners_and_elements,
                        children: Vec::new(),
                    };

                    let mut child_node_index = 0;
                    {
                        let mut new_acc = WebWriterAcc {
                            system_ptr: acc.system_ptr,
                            keys: keys,
                            document: acc.document,
                            parent_element: &vnode.web_element,
                            node_level: &mut vnode.children,
                            node_index: &mut child_node_index,
                        };
                        node.children().process_all::<WebWriter<D, U, R, S>>(&mut new_acc)?;
                    }
                    // Remove DOM elements left over from the last render that weren't repurposed
                    while child_node_index < vnode.children.len() {
                        let unused_dom_element = vnode.children.pop().unwrap();
                        unused_dom_element.web_element.remove_self();
                    }

                    acc.parent_element.insert(*acc.node_index, &vnode.web_element);
                    acc.node_level.insert(*acc.node_index, vnode);
                }

                *acc.node_index += 1;
                Ok(())
            }

            add_node
        }
    }

    struct ListenersToVec<Message: 'static>(PhantomData<Message>);
    impl<'a, M: 'static> ListenerProcessor<'a, M> for ListenersToVec<M> {
        type Acc = Vec<*const Listener<Message=M>>;
        type Error = ();

        fn get_processor<L: Listener<Message=M>>() -> fn(&mut Self::Acc, &'a L) -> Result<(), Self::Error> {
            fn add_listener_to_vec<L: Listener>(
                vec: &mut Vec<*const Listener<Message=L::Message>>,
                listener: &L) -> Result<(), ()>
            {
                vec.push(
                    // Extend the lifetime of the listener (we know it's valid until at least the
                    // next callback) and convert it to a *const
                    unsafe { mem::transmute(listener as &Listener<Message=L::Message>) }
                );
                Ok(())
            }
            add_listener_to_vec
        }
    }
}
