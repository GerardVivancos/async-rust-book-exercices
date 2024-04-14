#![feature(naked_functions)]
use std::{arch::asm, char::MAX};

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}

struct Thread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

// https://developer.arm.com/documentation/102374/0102/Procedure-Call-Standard
// Callee-saved registers are X19-X28, plus Frame Pointer (X29). X30 is the Link Register which stores the return address.
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    x19: u64,
    x20: u64,
    x21: u64,
    x22: u64,
    x23: u64,
    x24: u64,
    x25: u64,
    x26: u64,
    x27: u64,
    x28: u64,
    x29: u64,
    x30: u64,
}

impl Thread {
    fn new() -> Self {
        Self {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }

    fn new_running() -> Self {
        let mut thread = Self::new();
        thread.state = State::Running;
        thread
    }
}

impl Runtime {
    pub fn new() -> Self {
        // Initialize `threads` with a base thread set to running
        let mut threads = vec![Thread::new_running()];

        // Initialize the rest of the threads which are actual threads our runtime has available
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0,
        }
    }

    /// Sets the global reference to RUNTIME to point to this instance of Runtime.
    /// This is a shortcut from the book to focus on building fibers instead of dealing with ownership.
    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {
            
        }
        std::process::exit(0)
    }

    /// Called when a thread is finished, this frees the thread setting it as available.
    /// Which thread is being worked on is defined by the `current` thread number in `Runtime`.
    /// Does nothing if the current thread is 0 (the base thread).
    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;
        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }

            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new: *const ThreadContext = &self.threads[pos].ctx;

            asm!("call switch", in("x0") old, in("x1") new, clobber_abi("C"));
        }

        self.threads.len()> 0
    }

    pub fn spawn(&mut self, f: fn()) {
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no available thread");

        let size = available.stack.len();
        unsafe {
            // TODO: Prepare Macos/ARM64 stack
        }
        available.state = State::Ready;
    }
}

fn main() {
    println!("Hello, world!");
}
