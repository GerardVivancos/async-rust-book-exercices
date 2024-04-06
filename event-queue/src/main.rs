use std::{io::{self, Read, Result, Write}, net::TcpStream};
use ffi::Event;
use poll::Poll;

mod ffi;
mod poll;


fn main() -> Result<()> {
    let mut poll = Poll::new()?;
    let num_events = 5;

    let mut streams = vec![];
    let addr = "localhost:8080";

    for i in 0..num_events {
        let delay = (num_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;

        stream.write_all(request.as_bytes())?;
        poll.registry().register(&stream, i, ffi::EPOLLIN | ffi::EPOLLET)?;

        streams.push(stream);
    }

    let mut handled_events = 0;
    while handled_events < num_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;

        if events.is_empty() {
            println!("TIMEOUT OR SOMETHING WEIRD");
            continue;
        }

        handled_events += handle_events(&events, &mut streams)?;
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

fn handle_events(events: &[Event], streams: &mut[TcpStream]) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];

        loop {
            match streams[index].read(&mut data) {
                Ok(n) if n == 0 => {
                    // we successfully read 0 bytes, hence we're done with the stream
                    handled_events += 1;
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

    Ok(handled_events)

}
