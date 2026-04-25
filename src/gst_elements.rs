use gst::prelude::*;
use gstreamer as gst;

use crate::gst_error::InnerError;

pub(super) trait ElementTrait {
    fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError>;
    fn set_state(&self, state: gst::State) -> Result<(), InnerError>;
    fn is_all_null(&self) -> bool;
}

pub(super) struct MainSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
}

impl ElementTrait for MainSrcElements {
    fn add_to_pipeline(&self, pipeline: &gstreamer::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many(self.get_elements())
            .map_err(InnerError::GlibBool)
    }

    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        Ok(())
    }

    fn is_all_null(&self) -> bool {
        for element in self.get_elements() {
            let state = element.current_state();
            if state != gst::State::Null {
                return false
            }
        }
        true
    }
}

impl MainSrcElements {
    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![&self.src, &self.caps, &self.queue]
    }
}

pub(super) struct DownSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
}

impl ElementTrait for DownSrcElements {
    fn add_to_pipeline(&self, pipeline: &gstreamer::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many(self.get_elements())
            .map_err(InnerError::GlibBool)
    }

    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        Ok(())
    }

    fn is_all_null(&self) -> bool {
        for element in self.get_elements() {
            let state = element.current_state();
            if state != gst::State::Null {
                return false
            }
        }
        true
    }
}

impl DownSrcElements {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.src, &self.caps, &self.queue])
            .map_err(InnerError::GlibBool)
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![&self.src, &self.caps, &self.queue]
    }
}

pub(super) struct Sink {
    pub(super) selector: gst::Element,
    pub(super) queue: gst::Element,
    pub(super) sink: gst::Element,
}

impl Sink {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many([&self.selector, &self.queue, &self.sink])
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
