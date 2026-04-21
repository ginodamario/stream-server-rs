#![allow(unused)]
use crossbeam::channel;
use gst::{MessageType, prelude::*};
use gstreamer as gst;
use std::thread::{self, JoinHandle};

use crate::gst_elements::{DownSrcElements, Elements, MainSrcElements, Sink};
use crate::gst_error::{Error, InnerError};

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

            let elements = Self::create_element().map_err(Error::Pipeline)?;

            let pipeline = gst::Pipeline::with_name("test-pipeline");
            elements
                .add_to_pipeline(&pipeline)
                .map_err(Error::Pipeline)?;

            let main = &elements.main;
            let down = &elements.down;
            let main_sink = &elements.main_sink;

            gst::Element::link_many([&main.src, &main.caps, &main.queue, &main.watchdog])
                .map_err(|e| Error::Init(InnerError::GlibBool(e)))?;
            gst::Element::link_many([&down.src, &down.caps, &down.queue, &down.watchdog])
                .map_err(|e| Error::Init(InnerError::GlibBool(e)))?;

            gst::Element::link_many([&main_sink.selector, &main_sink.sink])
                .map_err(|e| Error::Init(InnerError::GlibBool(e)))?;

            let sel_pad_0 =
                main_sink
                    .selector
                    .request_pad_simple("sink_%u")
                    .ok_or(Error::LinkStr(
                        "Linking request main select pad 0".to_string(),
                    ))?;
            let sel_pad_1 =
                main_sink
                    .selector
                    .request_pad_simple("sink_%u")
                    .ok_or(Error::LinkStr(
                        "Linking request main select pad 1".to_string(),
                    ))?;

            let main_watchdog_src = main
                .watchdog
                .static_pad("src")
                .ok_or(Error::LinkStr("Get main watchdog src pad".to_string()))?;
            let down_watchdog_src = down
                .watchdog
                .static_pad("src")
                .ok_or(Error::LinkStr("Get pip watchdog src pad".to_string()))?;

            main_watchdog_src
                .link(&sel_pad_0)
                .map_err(|e| Error::Link(InnerError::Link(e)))?;
            down_watchdog_src
                .link(&sel_pad_1)
                .map_err(|e| Error::Link(InnerError::Link(e)))?;

            main_sink.selector.set_property("active-pad", &sel_pad_0);

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
                            if let Some(obj) = err.src()
                                && let Some(element) = obj.downcast_ref::<gst::Element>()
                                && element.has_as_ancestor(&main.watchdog)
                            {
                                if element.has_as_ancestor(&main.watchdog) {
                                    println!("Watchdog Main Error");
                                } else if element.has_as_ancestor(&down.watchdog) {
                                    println!("Watchdog Down Error");
                                }
                                // Don't break.
                            } else {
                                eprintln!(
                                    "Error recieved from element {:?}: {}",
                                    err.src().map(|s| s.path_string()),
                                    err.error()
                                );
                                eprintln!("Debugging information: {:?}", err.debug());
                                break;
                            }
                        }
                        MessageView::Eos(_) => {
                            println!("eos");
                        }
                        _ => {}
                    },
                    None => {
                        let cmd = recv_to_thread.try_recv().unwrap_or(Cmd::None);
                        match cmd {
                            Cmd::None => {}
                            Cmd::Select(source) => {
                                let pad = match source {
                                    Source::Main => &sel_pad_0,
                                    Source::Down => &sel_pad_1,
                                };
                                elements.main_sink.selector.set_property("active-pad", pad);
                            }
                            Cmd::Stop(source) => match source {
                                Source::Main => elements.main.src.set_state(gst::State::Null),
                                Source::Down => todo!(),
                            },
                            Cmd::Start(source) => todo!(),
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

    fn create_element() -> Result<Elements, InnerError> {
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
        let queue = gst::ElementFactory::make("queue")
            .name("main_queue")
            .build()
            .map_err(InnerError::GlibBool)?;
        let watchdog = gst::ElementFactory::make("watchdog")
            .name("main_watchdog")
            .build()
            .map_err(InnerError::GlibBool)?;

        let main_elements = MainSrcElements {
            src,
            caps,
            queue,
            watchdog,
        };

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
        let queue = gst::ElementFactory::make("queue")
            .name("down_queue")
            .build()
            .expect("Could not create queue element.");
        let watchdog = gst::ElementFactory::make("watchdog")
            .name("down_watchdog")
            .build()
            .map_err(InnerError::GlibBool)?;

        let down_elements = DownSrcElements {
            src,
            caps,
            queue,
            watchdog,
        };

        let selector = gst::ElementFactory::make("input-selector")
            .name("selector")
            .build()
            .map_err(InnerError::GlibBool)?;
        let sink = gst::ElementFactory::make("autovideosink")
            .name("sink")
            .build()
            .map_err(InnerError::GlibBool)?;

        let main_sink = Sink { selector, sink };

        Ok(Elements {
            main: main_elements,
            down: down_elements,
            main_sink,
        })
    }
}
