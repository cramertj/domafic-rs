const KEY_STACK_LEN: u32 = 32;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Keys {
    pub size: u32,
    pub stack: [u32; KEY_STACK_LEN as usize],
}
impl Keys {
    /// Create a new `Keys` with no elements
    #[cfg_attr(not(target_os = "emscripten"), allow(dead_code))]
    pub fn new() -> Keys {
        Keys { size: 0, stack: [0; KEY_STACK_LEN as usize] }
    }

    /// Push a new key onto the `Keys`
    /// Immutable. Creates a new `Keys` with the top element.
    #[cfg_attr(not(target_os = "emscripten"), allow(dead_code))]
    pub fn push(&self, key: u32) -> Keys {
        let mut stack = self.stack; // Copied

        debug_assert!(
            self.size < KEY_STACK_LEN,
            "Only {} elements fit on a `Keys`. Your structure may be too deep.",
            KEY_STACK_LEN
        );

        stack[self.size as usize] = key;
        Keys { size: self.size + 1, stack: stack }
    }
}

pub struct KeyIter(Keys, u32);

impl Iterator for KeyIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 < self.0.size {
            let result = Some(self.0.stack[self.1 as usize] as usize);
            self.1 += 1;
            result
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.0.size - self.1) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for KeyIter {}

impl IntoIterator for Keys {
    type Item = usize;
    type IntoIter = KeyIter;

    /// Returns an iterator over the keys from bottom to top
    fn into_iter(self) -> KeyIter {
        KeyIter(self, 0)
    }
}
