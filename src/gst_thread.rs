#![allow(unused)]
use crossbeam::channel;
use gst::{MessageType, prelude::*};
use gstreamer as gst;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crate::gst_elements::{DownSrcElements, ElementTrait, Elements, MainSrcElements, Sink};
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

            let elements = Self::create_elements().map_err(Error::Pipeline)?;

            let pipeline = gst::Pipeline::with_name("pipeline");
            elements
                .add_to_pipeline(&pipeline)
                .map_err(Error::Pipeline)?;

            let main = &elements.main;
            let down = &elements.down;
            let main_sink = &elements.main_sink;

            elements.link().map_err(Error::Link)?;

            main_sink
                .selector
                .set_property("active-pad", &main_sink.selector_sink_pad_0);

            let mut main_src_probe = GstProbe::new(&main.src);

            pipeline
                .set_state(gst::State::Playing)
                .map_err(|e| Error::StateChange(InnerError::StateChange(e)))?;

            let bus = pipeline.bus().ok_or(Error::Pipeline(InnerError::Bus))?;

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
                            let queue_src_pad = elements.main.queue.static_pad("src").unwrap();

                            if queue_src_pad.is_linked() && elements.main.is_all_null_state() {
                                println!("unlink");
                                elements.main_sink.selector.set_property(
                                    "active-pad",
                                    &elements.main_sink.selector_sink_pad_1,
                                );
                                elements.main.queue.unlink(&elements.main_sink.selector);
                            }
                        }

                        let cmd = recv_to_thread.try_recv().unwrap_or(Cmd::None);
                        match cmd {
                            Cmd::None => {}
                            Cmd::Select(source) => {
                                let pad = match source {
                                    Source::Main => &elements.main_sink.selector_sink_pad_0,
                                    Source::Down => &elements.main_sink.selector_sink_pad_1,
                                };
                                elements.main_sink.selector.set_property("active-pad", pad);
                            }
                            Cmd::Start(source) => match source {
                                Source::Main => {
                                    let queue_src_pad =
                                        elements.main.queue.static_pad("src").unwrap();
                                    if !queue_src_pad.is_linked() {
                                        println!("re-linking");
                                        queue_src_pad
                                            .link(&elements.main_sink.selector_sink_pad_0)
                                            .unwrap();
                                    }

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

    fn create_elements() -> Result<Elements, InnerError> {
        let src = gst::ElementFactory::make("videotestsrc")
            .name("main_src")
            .property_from_str("pattern", "smpte")
            .property_from_str("is-live", "true")
            .build()
            .map_err(InnerError::GlibBool)?;
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "NV12")
            .field("width", 1920)
            .field("height", 1080)
            .field("framerate", gst::Fraction::new(30, 1))
            .build();
        let caps = gst::ElementFactory::make("capsfilter")
            .property("caps", &caps)
            .build()
            .map_err(InnerError::GlibBool)?;
        let watchdog = gst::ElementFactory::make("watchdog")
            .name("main_watchdog")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue = gst::ElementFactory::make("queue")
            .name("main_queue")
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        let main_elements = MainSrcElements { src, caps, queue };

        let src = gst::ElementFactory::make("videotestsrc")
            .name("down_src")
            .property_from_str("pattern", "ball")
            .property_from_str("is-live", "true")
            .build()
            .expect("Could not create source element.");
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "NV12")
            .field("width", 1920)
            .field("height", 1080)
            .field("framerate", gst::Fraction::new(30, 1))
            .build();
        let caps = gst::ElementFactory::make("capsfilter")
            .property("caps", &caps)
            .build()
            .expect("Could not create caps element.");
        let watchdog = gst::ElementFactory::make("watchdog")
            .name("down_watchdog")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue = gst::ElementFactory::make("queue")
            .name("down_queue")
            .property_from_str("leaky", "downstream")
            .build()
            .expect("Could not create queue element.");

        let down_elements = DownSrcElements { src, caps, queue };

        let selector = gst::ElementFactory::make("input-selector")
            .name("selector")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue = gst::ElementFactory::make("queue")
            .name("queue")
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let sink = gst::ElementFactory::make("autovideosink")
            .name("sink")
            .build()
            .map_err(InnerError::GlibBool)?;

        let selector_sink_pad_0 =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 0".to_string(),
                ))?;

        let selector_sink_pad_1 =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 1".to_string(),
                ))?;

        let main_sink = Sink {
            selector,
            selector_sink_pad_0,
            selector_sink_pad_1,
            queue,
            sink,
        };

        Ok(Elements {
            main: main_elements,
            down: down_elements,
            main_sink,
        })
    }

    fn link_elements() {
        // TODO:
    }
}
