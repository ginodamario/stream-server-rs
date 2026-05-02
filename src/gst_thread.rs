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
                .set_property("active-pad", &main_sink.selector_sink_pad_main);
            pip_sink
                .selector
                .set_property("active-pad", &pip_sink.selector_sink_pad_main);

            let mut main_src_probe = GstProbe::new(&main.src);
            let mut down_src_probe = GstProbe::new(&down.src);

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
                            tracing::error!(
                                "Error recieved from element {:?}: {}",
                                err.src().map(|s| s.path_string()),
                                err.error()
                            );
                            tracing::error!("Debugging information: {:?}", err.debug());
                            break;
                        }
                        MessageView::Eos(_) => {
                            tracing::info!("eos");
                        }
                        _ => {}
                    },
                    None => {
                        if main_src_probe.is_stale() {
                            Self::handle_main_stopped(&elements);
                        }
                        if down_src_probe.is_stale() {
                            tracing::info!("down stale");
                        }

                        let cmd = recv_to_thread.try_recv().unwrap_or(Cmd::None);
                        match cmd {
                            Cmd::None => {}
                            Cmd::SelectMain(source) => {
                                let main_pad = match source {
                                    Source::Main => &elements.main_sink.selector_sink_pad_main,
                                    Source::Down => &elements.main_sink.selector_sink_pad_down,
                                };
                                elements
                                    .main_sink
                                    .selector
                                    .set_property("active-pad", main_pad);
                            }
                            Cmd::SelectPip(source) => {
                                let pip_pad = match source {
                                    Source::Main => &elements.pip_sink.selector_sink_pad_main,
                                    Source::Down => &elements.pip_sink.selector_sink_pad_down,
                                };
                                elements
                                    .pip_sink
                                    .selector
                                    .set_property("active-pad", pip_pad);
                            }
                            Cmd::Start(source) => match source {
                                Source::Main => {
                                    // todo!();
                                    // TODO Check if already running.
                                    elements.recreate_main(&pipeline).unwrap();
                                    // let queue_src_pad =
                                    //     elements.main.queue.static_pad("src").unwrap();
                                    // if !queue_src_pad.is_linked() {
                                    //     println!("re-linking");
                                    //     queue_src_pad
                                    //         .link(&elements.main_sink.selector_sink_pad_0)
                                    //         .unwrap();
                                    // }

                                    elements.main.set_state(gst::State::Playing).unwrap();
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
                                tracing::info!("exiting gst thread");
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

    fn handle_main_stopped(elements: &Elements) -> Result<(), InnerError> {
        // Stop all main elements.
        elements.main.set_state(gst::State::Null);

        let queue_0_main_src_pad = elements.main.queue_0_main_src.static_pad("src").unwrap();
        let queue_1_pip_src_pad = elements.main.queue_1_pip_src.static_pad("src").unwrap();

        if queue_0_main_src_pad.is_linked() {
            tracing::info!("unlink main to main_sink");
            let selected_pad: gst::Pad = elements.main_sink.selector.property("active-pad");
            if selected_pad == elements.main_sink.selector_sink_pad_main {
                elements
                    .main
                    .queue_0_main_src
                    .unlink(&elements.main_sink.selector);
            }
        }

        if queue_1_pip_src_pad.is_linked() {
            tracing::info!("unlink main to pip_sink");
            let selected_pad: gst::Pad = elements.pip_sink.selector.property("active-pad");
            if selected_pad == elements.pip_sink.selector_sink_pad_main {
                elements
                    .main
                    .queue_1_pip_src
                    .unlink(&elements.pip_sink.selector);
            }
        }

        Ok(())
    }
}
