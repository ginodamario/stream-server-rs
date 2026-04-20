use gst::prelude::*;
use gstreamer as gst;

use crate::gst_error::InnerError;

pub(super) struct MainSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
    pub(super) watchdog: gst::Element,
}

impl MainSrcElements {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.src, &self.caps, &self.queue, &self.watchdog])
            .map_err(InnerError::GlibBool)
    }
}

pub(super) struct DownSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
    pub(super) watchdog: gst::Element,
}

impl DownSrcElements {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.src, &self.caps, &self.queue, &self.watchdog])
            .map_err(InnerError::GlibBool)
    }
}

pub(super) struct Sink {
    pub(super) selector: gst::Element,
    pub(super) sink: gst::Element,
}

impl Sink {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.selector, &self.sink])
            .map_err(InnerError::GlibBool)?;
        Ok(())
    }
}

pub(super) struct Elements {
    pub(super) main: MainSrcElements,
    pub(super) down: DownSrcElements,
    pub(super) main_sink: Sink,
    // pub(super) pip_sink: Sink,
}

impl Elements {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.main.add_to_pipeline(pipeline)?;
        self.down.add_to_pipeline(pipeline)?;
        self.main_sink.add_to_pipeline(pipeline)?;
        // self.pip_sink.add_to_pipeline(pipeline)?;
        Ok(())
    }
}
