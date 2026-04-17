use anyhow::Result;
use gst_thread::GstThread;

mod gst_elements;
mod gst_error;
mod gst_thread;

fn main() -> Result<()> {
    let thread = GstThread::start();
    println!("Hello, world!");

    thread.join()?;

    Ok(())
}
