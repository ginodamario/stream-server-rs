#![allow(unused)]
use crossbeam::channel;
use gst::{MessageType, prelude::*};
use gstreamer as gst;
use std::thread::{self, JoinHandle};

use crate::gst_elements::{DownSrcElements, Elements, MainSrcElements};
use crate::gst_error::{Error, InnerError};

pub struct GstThread {
    handle: JoinHandle<Result<(), Error>>,
}

impl GstThread {
    pub fn start() -> Self {
        // let (send_from_thread, recv_from_thread) = channel::unbounded();
        // let (send_to_thread, recv_to_thread) = channel::unbounded();

        let handle = thread::spawn(move || {
            gst::init().map_err(|e| Error::Init(InnerError::GlibError(e)))?;

            let elements = Self::create_element().map_err(Error::CreatingPipeline)?;

            let pipeline = gst::Pipeline::with_name("test-pipeline");
            elements
                .add_to_pipeline(&pipeline)
                .map_err(Error::CreatingPipeline)?;

            let main = &elements.main;
            let down = &elements.down;
            let main_sink = &elements.main_sink;
            gst::Element::link_many([
                &main.src,
                &main.caps,
                &main.queue,
                &main.watchdog,
                &main_sink.select,
            ])
            .map_err(|e| Error::Init(InnerError::GlibBoolError(e)))?;
            gst::Element::link_many([
                &down.src,
                &down.caps,
                &down.queue,
                &down.watchdog,
                &main_sink.select,
            ])
            .map_err(|e| Error::Init(InnerError::GlibBoolError(e)))?;
            gst::Element::link_many([&main_sink.select, &main_sink.sink])
                .map_err(|e| Error::Init(InnerError::GlibBoolError(e)))?;
            // TODO: Request pad from selector instead.

            Ok(())
        });

        Self { handle }
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
            .map_err(InnerError::GlibBoolError)?;
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "NV12")
            .field("width", 1920)
            .field("height", 1080)
            .field("framerate", gst::Fraction::new(30, 1))
            .build();
        let caps = gst::ElementFactory::make("capsfilter")
            .property("caps", &caps)
            .build()
            .map_err(InnerError::GlibBoolError)?;
        let queue = gst::ElementFactory::make("queue")
            .name("main_queue")
            .build()
            .map_err(InnerError::GlibBoolError)?;
        let watchdog = gst::ElementFactory::make("watchdog")
            .name("main_watchdog")
            .build()
            .map_err(InnerError::GlibBoolError)?;

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
            .map_err(InnerError::GlibBoolError)?;

        let down_elements = DownSrcElements {
            src,
            caps,
            queue,
            watchdog,
        };

        Ok(Elements {
            main: main_elements,
            down: down_elements,
        })
    }
}
