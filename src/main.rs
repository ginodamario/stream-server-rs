use anyhow::Result;
use gst_thread::GstThread;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;

mod gst_elements;
mod gst_error;
mod gst_thread;

#[derive(Debug)]
enum SockCmd {
    Toggle,
}

fn main() -> Result<()> {
    let thread = GstThread::start();
    println!("Hello, world!");

    let path = "/tmp/stream-server.sock";
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path)?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf)?;

        let cmd = str::from_utf8(&buf[..n]).unwrap_or("");
        println!("cmd: {}", cmd.trim());

        let split: Vec<&str> = cmd.split_whitespace().collect();
        if split.len() == 2 {
            if split[0] == "sel" {
                if split[1] == "main" {
                    thread.send_cmd(gst_thread::Cmd::Select(gst_thread::Source::Main));
                } else if split[1] == "down" {
                    thread.send_cmd(gst_thread::Cmd::Select(gst_thread::Source::Down));
                }
            }
        }
        if cmd.starts_with("sel") {}
    }

    println!("joining threads");
    thread.join()?;

    Ok(())
}
