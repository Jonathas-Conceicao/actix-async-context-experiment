use std::collections::VecDeque;

pub(super) struct FutureSet<V> {
    set: VecDeque<V>,
}

impl<V> Default for FutureSet<V> {
    fn default() -> Self {
        FutureSet { set: VecDeque::default() }
    }
}

impl<V> FutureSet<V> {
    pub(super) fn insert(&mut self, v: V) -> usize {
        self.set.push_back(v);
        0
    }

    pub(super) fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.set.len()
    }

    pub(super) fn next(&self) -> Option<&V> {
        self.set.front()
    }

    pub(super) fn next_mut(&mut self) -> Option<&mut V> {
        self.set.front_mut()
    }

    pub(super) fn pop(&mut self) -> Option<V> {
        self.set.pop_front()
    }

    pub(super) fn append(&mut self, other: &mut Self) {
        self.set.append(&mut other.set)
    }
}
