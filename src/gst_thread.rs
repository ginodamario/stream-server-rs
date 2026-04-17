#![allow(unused)]
use crossbeam::channel;
use gst::{MessageType, prelude::*};
use gstreamer as gst;
use std::thread::{self, JoinHandle};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Init: {0}")]
    Init(InnerError),
    #[error("Creating Pipeline: {0}")]
    CreatingPipeline(InnerError),
    #[error("Join")]
    Join,
}

#[derive(Debug, Error)]
pub enum InnerError {
    #[error("gst: {0}")]
    GlibError(gstreamer::glib::Error),
    #[error("gst: {0}")]
    GlibBoolError(gstreamer::glib::BoolError),
}

struct MainSrcElements {
    src: gst::Element,
    caps: gst::Element,
    queue: gst::Element,
    watchdog: gst::Element,
}

impl MainSrcElements {
    fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.src, &self.caps, &self.queue, &self.watchdog])
            .map_err(InnerError::GlibBoolError)
    }
}

struct DownSrcElements {
    src: gst::Element,
    caps: gst::Element,
    queue: gst::Element,
    watchdog: gst::Element,
}

impl DownSrcElements {
    fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.src, &self.caps, &self.queue, &self.watchdog])
            .map_err(InnerError::GlibBoolError)
    }
}

struct Elements {
    main: MainSrcElements,
    down: DownSrcElements,
}

impl Elements {
    fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.main.add_to_pipeline(pipeline)?;
        self.down.add_to_pipeline(pipeline)?;
        Ok(())
    }
}

pub struct GstThread {
    handle: JoinHandle<Result<(), Error>>,
}

impl GstThread {
    pub fn start() -> Self {
        // let (send_from_thread, recv_from_thread) = channel::unbounded();
        // let (send_to_thread, recv_to_thread) = channel::unbounded();

        let handle = thread::spawn(move || {
            gst::init().map_err(|e| Error::Init(InnerError::GlibError(e)))?;

            let elements = Self::create_element().map_err(|e| Error::CreatingPipeline(e))?;

            let pipeline = gst::Pipeline::with_name("test-pipeline");
            elements
                .add_to_pipeline(&pipeline)
                .map_err(|e| Error::CreatingPipeline(e))?;

            let main = &elements.main;
            // gst::Element::link_many([main.src, main.caps, main.queue, main.watchdog, ])
            //     .expect("Elements could not be linked");

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
