// #[cfg(target_os = "linux")]
mod epoll;
pub use epoll::Event;

// #[cfg(target_os = "macos")]
// mod kqueue;

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
