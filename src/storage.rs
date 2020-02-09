use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

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

    pub(super) fn next(&self) -> Option<&V> {
        self.set.front()
    }

    pub(super) fn next_mut(&mut self) -> Option<&mut V> {
        self.set.front_mut()
    }

    pub(super) fn pop(&mut self) -> Option<V> {
        self.set.pop_front()
    }
}

impl<T> Deref for FutureSet<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<T> DerefMut for FutureSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}
