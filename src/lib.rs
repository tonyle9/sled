/// A lock-free log-structured b-link tree.

extern crate libc;
extern crate crossbeam;
extern crate rustc_serialize;
extern crate bincode;
extern crate time;
extern crate rand;

// transactional kv with multi-key ops
pub use db::DB;
// atomic lock-free tree
pub use tree::Tree;
// lock-free pagecache
pub use page::Pages;
// lock-free log-structured storage
pub use log::Log;
// lock-free stack
pub use stack::Stack;
// lock-free radix tree
pub use radix::Radix;

use crc16::crc16_arr;

macro_rules! rep_no_copy {
    ($e:expr; $n:expr) => {
        {
            let mut v = Vec::with_capacity($n);
            for _ in 0..$n {
                v.push($e);
            }
            v
        }
    };
}

macro_rules! read_or_break {
    ($file:expr, $buf:expr, $count:expr) => (
        match $file.read(&mut $buf) {
            Ok(n) if n == $buf.len() => {
                $count += n;
            },
            Ok(_) => {
                // tear occurred here
                break;
            },
            Err(_) => {
                break
            }
        }
    )
}

#[cfg(test)]
fn test_fail() -> bool {
    use rand::Rng;
    rand::thread_rng().gen::<bool>();
    // TODO when the time is right, return the gen'd bool
    false
}

#[cfg(not(test))]
#[inline(always)]
fn test_fail() -> bool {
    false
}

mod db;
mod tree;
mod bound;
mod log;
mod crc16;
mod stack;
mod page;
mod radix;

pub mod ops;

use bound::Bound;
use stack::{node_from_frag_vec, StackIter};
use page::{Frag, PageView, ChildSplit, ParentSplit};

type LogID = u64; // LogID == file position to simplify file mapping
type PageID = usize;

type Key = Vec<u8>;
type Value = Vec<u8>;

type Raw = *const stack::Node<*const page::Frag>;
type Page = Stack<*const page::Frag>;

#[inline(always)]
fn raw<T>(t: T) -> *const T {
    Box::into_raw(Box::new(t)) as *const T
}
