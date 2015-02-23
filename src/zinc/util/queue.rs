// Rough structure taken from libsync's mpcs_intrusive
// all links are usize to allow for static initialization

//
// head                       tail
// | |--->| |--->| |--->| |--->| |
//

use core::ty::Unsafe;
use core::cmp::Ord;
use core::ops::Deref;
use core::ptr::RawPtr;
use core::option::Option::{self, Some, None};

use hal::cortex_m3::sched::NoInterrupts;

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

  /// Push to tail.
  pub unsafe fn push(&self, node: *mut Node<T>, _: &NoInterrupts) {
    if (*self.head.get()).is_null() {
      *self.head.get() = node;
    }
    let tail: *mut Node<T> = *self.tail.get();
    *(*node).next.get() = null_mut();
    if !tail.is_null() {
      *(*tail).next.get() = node;
    }
    *self.tail.get() = node;
  }

  /// Peek at head.
  pub unsafe fn peek(&self) -> Option<*mut Node<T>> {
    let head = self.head.get();
    if (*head).is_null() {
      None
    } else {
      Some(*head)
    }
  }

  /// Pop off of head.
  pub unsafe fn pop(&self, _: &NoInterrupts) -> Option<*mut Node<T>> {
    let head = self.head.get();
    if (*head).is_null() {
      None
    } else {
      *head = *(**head).next.get();
      Some(*head)
    }
  }
}

impl<T: Ord> Queue<T> {
  /// Priority insertion (higher ends up closer to head).
  pub unsafe fn insert(&self, node: *mut Node<T>, _: &NoInterrupts) {
    let mut next: &Unsafe<*mut Node<T>> = &self.head;
    loop {
      let i: *mut Node<T> = *next.get();
      if i.is_null() {
        break;
      }
      if (*i).data > (*node).data {
        break;
      }
      next = &(*i).next;
    }
    *(*node).next.get() = *next.get();
    *next.get() = node;
    if (*(*node).next.get()).is_null() {
      *self.tail.get() = node;
    }
  }
}

impl<T> Node<T> {
  pub fn new(data: T) -> Node<T> {
    Node { next: Unsafe::new(null_mut()), data: data }
  }
}

impl<T> Deref<T> for Node<T> {
  fn deref<'a>(&'a self) -> &'a T {&self.data}
}
