use std::{io::{self, Result}, net::TcpStream, os::fd::AsRawFd};
use crate::ffi::{self, Event};

type Events = Vec<Event>;

pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Self> {
        let res = unsafe {
            ffi::epoll_create(1) // The argument is actually ignored by epoll_create, but must be > 0
        };

        // epoll_create returns -1 if there's an error, or the fd else
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
        let timeout = timeout.unwrap_or(-1); // -1: No timeout
        let max_events = events.capacity() as i32;
        
        // this will wait for events or `timeout` expiring
        // then write the events to `events`, and `res` will be the number of events written
        // we won't get more events than `max_events` (which we use the `events` vector capacity to define)
        let res = unsafe { ffi::epoll_wait(fd, events.as_mut_ptr(), max_events, timeout) };

        // `epoll_wait` returns -1 if there's an error, or the number of events else
        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        // `epoll_wait` wrote `res` number of events to the `events` vector memory space.
        // Here we tell Rust that the number of valid items in the vector is `res`.
        // This is safe because we trust the OS 
        unsafe { events.set_len(res as usize) };
        Ok(())
    }
}

pub struct Registry {
    raw_fd: i32,
}

impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        let mut event = ffi::Event {
            events: interests as u32,
            epoll_data: token,
        };

        let op = ffi::EPOLL_CTL_ADD;
        let res = unsafe {
            ffi::epoll_ctl(self.raw_fd, op, source.as_raw_fd(), &mut event)
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
