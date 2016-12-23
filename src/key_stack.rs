const KEY_STACK_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct KeyStack {
    size: usize,
    stack: [usize; KEY_STACK_LEN]
}
impl KeyStack {
    /// Create a new `KeyStack` with no elements
    pub fn new() -> KeyStack {
        KeyStack { size: 0, stack: [0; KEY_STACK_LEN] }
    }

    /// Get the `index`th element pushed onto the stack
    pub fn get_at(&self, index: usize) -> Option<usize> {
        if index < self.size {
            Some(self.stack[index])
        } else {
            None
        }
    }

    /// Push a new key onto the `KeyStack`
    /// Immutable. Creates a new `KeyStack` with the top element.
    pub fn push(&self, key: usize) -> KeyStack {
        let mut stack = self.stack.clone();

        debug_assert!(
            self.size < KEY_STACK_LEN,
            "Only {} elements fit on a `KeyStack`", KEY_STACK_LEN);

        stack[self.size] = key;
        KeyStack { size: self.size + 1, stack: stack }
    }

    /// Pop a new key off of the `KeyStack`
    /// Immutable. Creates a new `KeyStack` without the top element.
    pub fn pop(&self) -> (KeyStack, usize) {
        debug_assert!(self.size > 0, "Cannot pop from an empty KeyStack");
        (
            KeyStack { size: self.size - 1, stack: self.stack.clone() },
            self.stack[self.size - 1]
        )
    }

    /// Retrieves the first element pushed onto the stack
    pub fn bottom(&self) -> usize {
        debug_assert!(self.size > 0, "Cannot take bottom of empty stack");
        self.stack[0]
    }

    /// Iterates over the elements from first pushed to last pushed
    pub fn iter_from_bottom<'a>(&'a self) -> KeyStackFromBottomIter<'a> {
        KeyStackFromBottomIter {
            stack: self,
            iter_index: 0,
        }
    }
}

pub struct KeyStackFromBottomIter<'a> {
    stack: &'a KeyStack,
    iter_index: usize,
}

impl<'a> Iterator for KeyStackFromBottomIter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if let item @ Some(_) = self.stack.get_at(self.iter_index) {
            self.iter_index += 1;
            item
        } else {
            None
        }
    }
}
