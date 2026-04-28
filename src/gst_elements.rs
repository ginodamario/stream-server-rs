use gst::prelude::*;
use gstreamer as gst;

use crate::gst_error::InnerError;

pub(super) trait ElementTrait {
    fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
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

    fn is_all_null_state(&self) -> bool {
        for element in self.get_elements() {
            let state = element.current_state();
            if state != gst::State::Null {
                return false;
            }
        }
        true
    }

    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many(self.get_elements()).map_err(InnerError::GlibBool)?;

        Ok(())
    }

    fn get_last_element(&self) -> Result<&gst::Element, InnerError> {
        let e = *self.get_elements().last().ok_or(InnerError::GetElement)?;
        Ok(e)
    }

    fn get_elements(&self) -> Vec<&gst::Element>;
}

pub(super) struct MainSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
}

impl ElementTrait for MainSrcElements {
    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![&self.src, &self.caps, &self.queue]
    }
}

impl MainSrcElements {}

pub(super) struct DownSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
}

impl ElementTrait for DownSrcElements {
    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![&self.src, &self.caps, &self.queue]
    }
}

impl DownSrcElements {}

pub(super) struct Sink {
    pub(super) selector: gst::Element,
    pub(super) selector_sink_pad_0: gst::Pad,
    pub(super) selector_sink_pad_1: gst::Pad,
    pub(super) queue: gst::Element,
    pub(super) sink: gst::Element,
}

impl ElementTrait for Sink {
    fn get_elements(&self) -> Vec<&gstreamer::Element> {
        vec![&self.selector, &self.queue, &self.sink]
    }
}

impl Sink {}

pub(super) struct Elements {
    pub(super) main: MainSrcElements,
    pub(super) down: DownSrcElements,
    pub(super) main_sink: Sink,
}

impl Elements {
    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.main.add_to_pipeline(pipeline)?;
        self.down.add_to_pipeline(pipeline)?;
        self.main_sink.add_to_pipeline(pipeline)?;
        Ok(())
    }

    pub(super) fn link(&self) -> Result<(), InnerError> {
        self.main.link()?;
        self.down.link()?;
        self.main_sink.link()?;

        self.link_element_to_sink_pad(
            self.main.get_last_element()?,
            &self.main_sink.selector_sink_pad_0,
        )?;
        self.link_element_to_sink_pad(
            self.down.get_last_element()?,
            &self.main_sink.selector_sink_pad_1,
        )?;

        Ok(())
    }

    fn link_element_to_sink_pad(
        &self,
        src: &gst::Element,
        pad: &gst::Pad,
    ) -> Result<(), InnerError> {
        let src_pad = src
            .static_pad("src")
            .ok_or(InnerError::RequestPad("Get main queu src pad".to_string()))?;

        src_pad.link(pad).map_err(InnerError::Link)?;

        Ok(())
    }
}
