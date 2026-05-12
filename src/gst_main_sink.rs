use gst::prelude::*;
use gstreamer as gst;

use crate::gst_element_trait::ElementTrait;
use crate::gst_error::InnerError;

pub(crate) struct MainSink {
    pub(crate) selector: gst::Element,
    pub(crate) selector_sink_pad_main: gst::Pad,
    pub(crate) selector_sink_pad_down: gst::Pad,
    pub(crate) queue: gst::Element,
    pub(crate) sink: gst::Element,
}

impl ElementTrait for MainSink {
    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gstreamer::Element> {
        vec![&self.selector, &self.queue, &self.sink]
    }
}

impl MainSink {
    pub(crate) fn new() -> Result<Self, InnerError> {
        let selector = gst::ElementFactory::make("input-selector")
            .name("selector")
            .property("sync-streams", false)
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

        let selector_sink_pad_main =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 0".to_string(),
                ))?;

        let selector_sink_pad_down =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 1".to_string(),
                ))?;

        Ok(MainSink {
            selector,
            selector_sink_pad_main,
            selector_sink_pad_down,
            queue,
            sink,
        })
    }
}
