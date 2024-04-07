#[cfg(target_os = "linux")] 
mod epoll;
#[cfg(target_os = "linux")]
pub use epoll::Event;

#[cfg(target_os = "macos")]
mod kqueue;
#[cfg(target_os = "macos")]
pub use kqueue::Event;

type Events = Vec<Event>;

pub enum Interest {
    READ,
}

pub struct Poll {
    registry: Registry,
}

pub struct Registry {
    raw_fd: i32,
}
