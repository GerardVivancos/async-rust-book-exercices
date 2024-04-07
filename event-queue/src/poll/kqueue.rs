use std::{io::{self, Result}, net::TcpStream, os::fd::AsRawFd};
use super::{Events, Interest, Poll, Registry};
use std::ptr;

// https://man.freebsd.org/cgi/man.cgi?kqueue
// https://github.com/rust-lang/libc/blob/a0f5b4b21391252fe38b2df9310dc65e37b07d9f/src/unix/bsd/apple/mod.rs#L421
#[derive(Debug)]
#[repr(C, packed)]
pub struct Event {
    pub  ident:  u64,     /* identifier for this event */
    pub	 filter: i16,     /* filter for event */
    pub	 flags:  u16,     /* action flags for kqueue */
    pub	 fflags: u32,     /* filter flag value */
    pub	 data:   i64,     /* filter data value */
    pub	 udata:  u64,     /* opaque user data identifier */
    pub	 ext:    [u64;2], /* extensions */
}

impl Event {
    pub fn token(&self) -> usize {
        self.udata as usize
    }
}

impl Poll {
    pub fn new() -> Result<Self> {
        let res = unsafe {
            ffi::kqueue()
        };

        // `kqueue` returns -1 if there's an error, or the fd else
        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self{
            registry: Registry { raw_fd: res },
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        let fd = self.registry.raw_fd;
        // TODO: Enable timeout support
        let _timeout = timeout.unwrap_or(-1); // -1: No timeout
        let max_events = events.capacity() as i32;
        
        // this will wait for events or `timeout` expiring
        // then write the events to `events`, and `res` will be the number of events written
        // we won't get more events than `max_events` (which we use the `events` vector capacity to define)
        let res = unsafe { ffi::kevent(fd, ptr::null(), 0,events.as_mut_ptr(), max_events, ptr::null()) };

        // `kevent` returns -1 if there's an error, or the number of events else
        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        // `kevent` wrote `res` number of events to the `events` vector memory space.
        // Here we tell Rust that the number of valid items in the vector is `res`.
        // This is safe because we trust the OS 
        unsafe { events.set_len(res as usize) };
        Ok(())
    }
}

impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, _interest: Interest) -> Result<()> {
        // TODO: Enable interest selection
        let event = Event {
            ident: source.as_raw_fd() as u64,
            filter: ffi::EVFILT_READ,
            flags: ffi::EV_ADD | ffi::EV_ONESHOT | ffi::EV_ENABLE,
            fflags: 0,
            data: 0,
            udata: token as u64,
            ext: [0, 2],
        };

        let events = [event];

        let res = unsafe {
            ffi::kevent(self.raw_fd,
                events.as_ptr(),
                events.len().try_into().unwrap(),
                ptr::null_mut(),
                0,
                ptr::null(),
            )
        };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let res = unsafe {
            ffi::close(self.raw_fd)
        };

        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("ERROR: {err:?}");
        }
    }
}

#[allow(dead_code)]
mod ffi {
    // https://github.com/rust-lang/libc/blob/a0f5b4b21391252fe38b2df9310dc65e37b07d9f/src/unix/bsd/apple/mod.rs#L4264-L4278
    pub const EV_ADD: u16 = 0x1;
    pub const EV_DELETE: u16 = 0x2;
    pub const EV_ENABLE: u16 = 0x4;
    pub const EV_DISABLE: u16 = 0x8;
    pub const EV_ONESHOT: u16 = 0x10;
    pub const EV_CLEAR: u16 = 0x20;
    pub const EV_RECEIPT: u16 = 0x40;
    pub const EV_DISPATCH: u16 = 0x80;
    pub const EV_FLAG0: u16 = 0x1000;
    pub const EV_POLL: u16 = 0x1000;
    pub const EV_FLAG1: u16 = 0x2000;
    pub const EV_OOBAND: u16 = 0x2000;
    pub const EV_ERROR: u16 = 0x4000;
    pub const EV_EOF: u16 = 0x8000;
    pub const EV_SYSFLAGS: u16 = 0xf000;

    // https://github.com/rust-lang/libc/blob/a0f5b4b21391252fe38b2df9310dc65e37b07d9f/src/unix/bsd/apple/mod.rs#L4252-L4262C28
    pub const EVFILT_READ: i16 = -1;
    pub const EVFILT_WRITE: i16 = -2;
    pub const EVFILT_AIO: i16 = -3;
    pub const EVFILT_VNODE: i16 = -4;
    pub const EVFILT_PROC: i16 = -5;
    pub const EVFILT_SIGNAL: i16 = -6;
    pub const EVFILT_TIMER: i16 = -7;
    pub const EVFILT_MACHPORT: i16 = -8;
    pub const EVFILT_FS: i16 = -9;
    pub const EVFILT_USER: i16 = -10;
    pub const EVFILT_VM: i16 = -12;

    use super::Event;

    #[link(name="c")]
    extern "C" {
        pub fn kqueue() -> i32;
        pub fn kevent(kq: i32, changelist: *const Event, nchanges: i32, eventlist: *mut Event, nevents: i32, timespec: *const Timespec) -> i32;
        pub fn close(d: i32) -> i32;
    }

    // https://github.com/rust-lang/libc/blob/a0f5b4b21391252fe38b2df9310dc65e37b07d9f/src/unix/mod.rs#L70C1-L76C6
    #[repr(C)]
    pub struct Timespec {
        pub tv_sec: isize,
        pub tv_nsec: usize,
    }
}
