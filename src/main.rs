use anyhow::Result;
use gst_thread::GstThread;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod gst_elements;
mod gst_error;
mod gst_probe;
mod gst_thread;

fn get_log_dir(pkg_name: &str) -> PathBuf {
    let fallback_log_dir = PathBuf::from("./");
    if let Some(dir) = dirs::state_dir() {
        let dir = dir.join(pkg_name);
        if let Ok(exist) = dir.try_exists() {
            if !exist && std::fs::create_dir_all(&dir).is_err() {
                tracing::warn!("Unable to create state directory. Using fallback log directory.");
                fallback_log_dir
            } else {
                dir
            }
        } else {
            tracing::warn!(
                "Unable to determine if state directory exists. Using fallback log directory."
            );
            fallback_log_dir
        }
    } else {
        tracing::warn!("Unable to local state directory. Using fallback log directory");
        fallback_log_dir
    }
}

fn main() -> Result<()> {
    let pkg_name = env!("CARGO_PKG_NAME");
    let log_dir = get_log_dir(pkg_name);
    let file_appender = rolling::never(log_dir, format!("{pkg_name}.log"));
    let (non_blocking, _guard_tracing) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(non_blocking)) // file output
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    tracing::info!("STARTING");

    let thread = GstThread::start();

    let path = "/tmp/stream-server.sock";
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path)?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf)?;

        let cmd = str::from_utf8(&buf[..n]).unwrap_or("");
        tracing::info!("cmd: {}", cmd.trim());

        let split: Vec<&str> = cmd.split_whitespace().collect();
        if split.len() == 2 {
            if split[0] == "selmain" {
                if split[1] == "main" {
                    thread.send_cmd(gst_thread::Cmd::SelectMain(gst_thread::Source::Main));
                } else if split[1] == "down" {
                    thread.send_cmd(gst_thread::Cmd::SelectMain(gst_thread::Source::Down));
                }
            } else if split[0] == "selpip" {
                if split[1] == "main" {
                    thread.send_cmd(gst_thread::Cmd::SelectPip(gst_thread::Source::Main));
                } else if split[1] == "down" {
                    thread.send_cmd(gst_thread::Cmd::SelectPip(gst_thread::Source::Down));
                }
            } else if split[0] == "start" {
                if split[1] == "main" {
                    thread.send_cmd(gst_thread::Cmd::Start(gst_thread::Source::Main));
                } else if split[1] == "down" {
                    thread.send_cmd(gst_thread::Cmd::Start(gst_thread::Source::Down));
                }
            } else if split[0] == "stop" {
                if split[1] == "main" {
                    thread.send_cmd(gst_thread::Cmd::Stop(gst_thread::Source::Main));
                } else if split[1] == "down" {
                    thread.send_cmd(gst_thread::Cmd::Stop(gst_thread::Source::Down));
                }
            }
        } else if split.len() == 1 {
            if split[0] == "exit" {
                thread.send_cmd(gst_thread::Cmd::Exit);
                break;
            }
        }
    }

    tracing::info!("joining threads");
    thread.join()?;

    Ok(())
}
