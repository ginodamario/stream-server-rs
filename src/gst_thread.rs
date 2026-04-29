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
use crate::gst_probe::GstProbe;

pub enum Source {
    Main,
    Down,
}

pub enum Cmd {
    None,
    Select(Source),
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

            let mut elements = Elements::new().map_err(Error::CreatePipeline)?;

            let pipeline = gst::Pipeline::with_name("pipeline");
            elements
                .add_to_pipeline(&pipeline)
                .map_err(Error::CreatePipeline)?;

            let main = &elements.main;
            let down = &elements.down;
            let main_sink = &elements.main_sink;
            let pip_sink = &elements.pip_sink;

            elements.link().map_err(Error::Link)?;

            main_sink
                .selector
                .set_property("active-pad", &main_sink.selector_sink_pad_0);
            pip_sink
                .selector
                .set_property("active-pad", &pip_sink.selector_sink_pad_0);

            let mut main_src_probe = GstProbe::new(&main.src);

            pipeline
                .set_state(gst::State::Playing)
                .map_err(|e| Error::StateChange(InnerError::StateChange(e)))?;

            let bus = pipeline
                .bus()
                .ok_or(Error::CreatePipeline(InnerError::Bus))?;

            while (true) {
                let msg = bus.timed_pop_filtered(
                    gst::ClockTime::from_mseconds(100),
                    &[MessageType::Error, MessageType::Eos],
                );

                use gst::MessageView;
                match msg {
                    Some(msg) => match msg.view() {
                        MessageView::Error(err) => {
                            eprintln!(
                                "Error recieved from element {:?}: {}",
                                err.src().map(|s| s.path_string()),
                                err.error()
                            );
                            eprintln!("Debugging information: {:?}", err.debug());
                            break;
                        }
                        MessageView::Eos(_) => {
                            println!("eos");
                        }
                        _ => {}
                    },
                    None => {
                        if main_src_probe.is_stale() {
                            println!("main stale");
                            // let queue_src_pad = elements.main.queue.static_pad("src").unwrap();
                            //
                            // if queue_src_pad.is_linked() && elements.main.is_all_null_state() {
                            //     println!("unlink");
                            //     elements.main_sink.selector.set_property(
                            //         "active-pad",
                            //         &elements.main_sink.selector_sink_pad_1,
                            //     );
                            //     elements.main.queue.unlink(&elements.main_sink.selector);
                            // }
                        }

                        let cmd = recv_to_thread.try_recv().unwrap_or(Cmd::None);
                        match cmd {
                            Cmd::None => {}
                            Cmd::Select(source) => {
                                // let pad = match source {
                                //     Source::Main => &elements.main_sink.selector_sink_pad_0,
                                //     Source::Down => &elements.main_sink.selector_sink_pad_1,
                                // };
                                // elements.main_sink.selector.set_property("active-pad", pad);
                                let pad = match source {
                                    Source::Main => &elements.pip_sink.selector_sink_pad_0,
                                    Source::Down => &elements.pip_sink.selector_sink_pad_1,
                                };
                                elements.pip_sink.selector.set_property("active-pad", pad);
                            }
                            Cmd::Start(source) => match source {
                                Source::Main => {
                                    todo!();
                                    // TODO Check if already running.
                                    // elements.recreate_main(&pipeline).unwrap();
                                    // let queue_src_pad =
                                    //     elements.main.queue.static_pad("src").unwrap();
                                    // if !queue_src_pad.is_linked() {
                                    //     println!("re-linking");
                                    //     queue_src_pad
                                    //         .link(&elements.main_sink.selector_sink_pad_0)
                                    //         .unwrap();
                                    // }

                                    // elements.main.set_state(gst::State::Playing).unwrap();
                                }
                                Source::Down => {
                                    let _ = elements.down.src.set_state(gst::State::Playing);
                                }
                            },
                            Cmd::Stop(source) => match source {
                                Source::Main => {
                                    elements.main.set_state(gst::State::Null).unwrap();
                                }
                                Source::Down => {
                                    elements.down.set_state(gst::State::Null).unwrap();
                                }
                            },
                            Cmd::Exit => {
                                // Ignore any error just exit.
                                let _ = pipeline.set_state(gst::State::Null);
                                println!("Exit");
                                break;
                            }
                        }
                    }
                }
            }
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
