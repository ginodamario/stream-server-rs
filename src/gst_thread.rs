#![allow(unused)]
use crossbeam::channel;
use gst::{MessageType, prelude::*};
use gstreamer as gst;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::gst_elements::{DownSrcElements, ElementTrait, Elements, MainSink, MainSrcElements};
use crate::gst_error::{Error, InnerError};
use crate::gst_pipeline::Pipeline;
use crate::gst_probe::GstProbe;
use crate::gst_source::Source;

pub enum Cmd {
    None,
    SelectMain(Source),
    SelectPip(Source),
    Stop(Source),
    Start(Source),
    Exit,
}

pub struct GstThread {
    handle: JoinHandle<Result<(), Error>>,
    send_to_thread: channel::Sender<Cmd>,
}

impl GstThread {
    pub fn start() -> Self {
        // let (send_from_thread, recv_from_thread) = channel::unbounded();
        let (send_to_thread, recv_to_thread) = channel::unbounded();

        let handle = thread::spawn(move || {
            gst::init().map_err(|e| Error::Init(InnerError::Glib(e)))?;

            let mut pipeline = Pipeline::new().unwrap();

            pipeline.switch_main_sink(Source::Main);
            pipeline.switch_pip_sink(Source::Down);

            pipeline.set_state(gst::State::Playing);

            pipeline.run_loop(|s| {
                let cmd = recv_to_thread.try_recv().unwrap_or(Cmd::None);
                match cmd {
                    Cmd::None => {}
                    Cmd::SelectMain(source) => {
                        s.switch_main_sink(source);
                    }
                    Cmd::SelectPip(source) => {
                        s.switch_pip_sink(source);
                    }
                    Cmd::Stop(source) => match source {
                        Source::Main => {
                            s.simulate_main_stop();
                        }
                        Source::Down => {
                            s.simulate_down_stop();
                        }
                    },
                    Cmd::Start(source) => match source {
                        Source::Main => {
                            // TODO Check if already running.
                            s.recreate_main();
                            s.set_main_state(gst::State::Playing);
                        }
                        Source::Down => {
                            s.recreate_down();
                            s.set_down_state(gst::State::Playing);
                        }
                    },
                    Cmd::Exit => {
                        // Ignore any error just exit.
                        s.set_state(gst::State::Null);
                        tracing::info!("exiting gst thread");
                        return false;
                    }
                    _ => {}
                }

                true
            });

            // while (true) {
            //     let msg = bus.timed_pop_filtered(
            //         gst::ClockTime::from_mseconds(100),
            //         &[MessageType::Error, MessageType::Eos],
            //     );
            //
            //     use gst::MessageView;
            //     match msg {
            //         Some(msg) => match msg.view() {
            //             MessageView::Error(err) => {
            //                 tracing::error!(
            //                     "Error recieved from element {:?}: {}",
            //                     err.src().map(|s| s.path_string()),
            //                     err.error()
            //                 );
            //                 tracing::error!("Debugging information: {:?}", err.debug());
            //                 break;
            //             }
            //             MessageView::Eos(_) => {
            //                 tracing::info!("eos");
            //             }
            //             _ => {}
            //         },
            //         None => {
            //             // if main_src_probe.is_stale() {
            //             //     Self::handle_main_stopped(&mut elements);
            //             // }
            //             // if down_src_probe.is_stale() {
            //             //     tracing::info!("down stale");
            //             // }
            //
            //             let cmd = recv_to_thread.try_recv().unwrap_or(Cmd::None);
            //             match cmd {
            //                 Cmd::Start(source) => match source {
            //                     Source::Main => {
            //                         // todo!();
            //                         // TODO Check if already running.
            //                         elements.recreate_main(&pipeline).unwrap();
            //                         // let queue_src_pad =
            //                         //     elements.main.queue.static_pad("src").unwrap();
            //                         // if !queue_src_pad.is_linked() {
            //                         //     println!("re-linking");
            //                         //     queue_src_pad
            //                         //         .link(&elements.main_sink.selector_sink_pad_0)
            //                         //         .unwrap();
            //                         // }
            //
            //                         elements.main.set_state(gst::State::Playing).unwrap();
            //                     }
            //                     Source::Down => {
            //                         let _ = elements.down.src.set_state(gst::State::Playing);
            //                     }
            //                 },
            //             }
            //         }
            //     }
            // }
            Ok(())
        });
        Self {
            handle,
            send_to_thread,
        }
    }

    pub fn send_cmd(&self, cmd: Cmd) {
        let _ = self.send_to_thread.send(cmd);
    }

    pub fn join(self) -> Result<(), Error> {
        self.handle.join().map_err(|_| Error::Join)?
    }
}
