// Rough structure taken from libsync's mpcs_intrusive
// all links are uint to allow for static initialization

use hal::cortex_m3::sched::IrqDisabled;
use std::cell::Cell;
use std::option::{Option,Some,None};

pub struct Node<T> {
    pub next: Cell<*mut Node<T>>,
    pub data: T
}

pub struct Queue<T> {
    pub head: Cell<*mut Node<T>>,
    pub tail: Cell<*mut Node<T>>
}

fn null_mut<T>() -> *mut T { 0 as *mut T }

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            head: Cell::new(null_mut()),
            tail: Cell::new(null_mut())
        }
    }

    /// Push to head
    pub unsafe fn push(&self, node: *mut Node<T>, _: IrqDisabled) {
        if self.head.get() == (0 as *mut Node<T>) {
            self.tail.set(node);
        }
        (*node).next.set(0 as *mut Node<T>);
        self.head.set(node);
    }

    pub unsafe fn peek(&self) -> Option<*mut Node<T>> {
        let a = self.tail.get();
        if a == null_mut() {
            None
        } else {
            Some(a)
        }
    }

    /// Pop off of tail
    pub unsafe fn pop(&self, _: IrqDisabled) -> Option<*mut Node<T>> {
        let a = self.tail.get();
        if a == null_mut() {
            None
        } else {
            self.tail.set((*self.tail.get()).next.get());
            Some(a)
        }
    }
}

impl<T> Node<T> {
    pub fn new(data: T) -> Node<T> {
        Node { next: Cell::new(null_mut()), data: data }
    }
}
