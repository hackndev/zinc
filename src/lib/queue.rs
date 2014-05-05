// Rough structure taken from libsync's mpcs_intrusive
// all links are uint to allow for static initialization

use hal::cortex_m3::sched::CritSection;
use std::ty::Unsafe;
use std::option::{Option,Some,None};

pub struct Node<T> {
    pub next: Unsafe<*mut Node<T>>,
    pub data: T
}

pub struct Queue<T> {
    pub head: Unsafe<*mut Node<T>>,
    pub tail: Unsafe<*mut Node<T>>
}

fn null_mut<T>() -> *mut T { 0 as *mut T }

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            head: Unsafe::new(null_mut()),
            tail: Unsafe::new(null_mut())
        }
    }

    /// Push to head
    pub unsafe fn push(&self, node: *mut Node<T>, _: &CritSection) {
        if *self.head.get() == (0 as *mut Node<T>) {
            *self.tail.get() = node;
        }
        *(*node).next.get() = 0 as *mut Node<T>;
        *self.head.get() = node;
    }

    /// Peek at tail
    pub unsafe fn peek(&self) -> Option<*mut Node<T>> {
        let tail = self.tail.get();
        if *tail == null_mut() {
            None
        } else {
            Some(*tail)
        }
    }

    /// Pop off of tail
    pub unsafe fn pop(&self, _: &CritSection) -> Option<*mut Node<T>> {
        let tail = self.tail.get();
        if *tail == null_mut() {
            None
        } else {
            *tail = *(**tail).next.get();
            Some(*tail)
        }
    }
}

impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node { next: Unsafe::new(null_mut()), data: data }
    }
}
