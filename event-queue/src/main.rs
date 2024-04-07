use std::{io::{self, Read, Result, Write}, net::TcpStream};
use poll::{Event, Poll};

mod poll;

#[derive(Debug)]
struct Streams {
    streams: Vec<TcpStream>,
    handled_streams: Vec<bool>,
}

impl Streams {
    fn new() -> Self {
        Self {
            streams: vec![],
            handled_streams: vec![],
        }
    }

    fn push(&mut self, stream: TcpStream) {
        self.streams.push(stream);
        self.handled_streams.push(false);
    }

    fn is_handled(&self, index: usize) -> bool {
        if self.handled_streams.len() < index + 1 {
            return false;
        }
        self.handled_streams[index]
    }

    fn all_handled(&self) -> bool {
        self.handled_streams.iter().all(|&x| x)
    }

    fn stream(&mut self, index: usize) -> &mut TcpStream {
        &mut self.streams[index]
    }

    fn complete(&mut self, index: usize) {
        self.handled_streams[index] = true;
    }
}

fn main() -> Result<()> {
    let mut poll = Poll::new()?;
    let num_events = 5;

    let mut streams = Streams::new();
    let addr = "localhost:8080";

    for i in 0..num_events {
        let delay = (num_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;

        stream.write_all(request.as_bytes())?;
        //TODO: Remove dependency on epoll
        poll.registry().register(&stream, i, poll::Interest::READ)?;

        streams.push(stream);
    }

    while !streams.all_handled() {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;

        if events.is_empty() {
            println!("TIMEOUT OR SOMETHING WEIRD");
            continue;
        }

        handle_events(&events, &mut streams)?;
    }

    println!("FINISHED");
    Ok(())
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
        HOST: localhost\r\n\
        Connection: close\r\n\
        \r\n"
    )
}

fn handle_events(events: &[Event], streams: &mut Streams) -> Result<()> {
    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];

        loop {
            if streams.is_handled(index) {
                break;
            };
            match streams.stream(index).read(&mut data) {
                Ok(n) if n == 0 => {
                    // we successfully read 0 bytes, hence we're done with the stream
                    streams.complete(index);
                    println!("read 0 for event {index}\n-----\n");
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("Received: {:?}", event);
                    println!("{txt}\n-----\n");
                }
                // Apparently we can be told the buffer is ready but it not being the case
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

    }
    Ok(())
}
