use gst::prelude::*;
use gstreamer as gst;

use crate::gst_elements::ElementTrait;
use crate::gst_error::InnerError;

pub(crate) struct PipSink {
    pub(crate) selector: gst::Element,
    pub(crate) selector_sink_pad_main: gst::Pad,
    pub(crate) selector_sink_pad_down: gst::Pad,
    pub(crate) video_scale: gst::Element,
    pub(crate) video_convert_caps: gst::Element,
    pub(crate) queue: gst::Element,
    pub(crate) sink: gst::Element,
}

impl ElementTrait for PipSink {
    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gstreamer::Element> {
        vec![
            &self.selector,
            &self.video_scale,
            &self.video_convert_caps,
            &self.queue,
            &self.sink,
        ]
    }
}

impl PipSink {
    pub(crate) fn new() -> Result<Self, InnerError> {
        let selector = gst::ElementFactory::make("input-selector")
            .name("pip_selector")
            .property("sync-streams", false)
            .build()
            .map_err(InnerError::GlibBool)?;
        let selector_sink_pad_main =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request pip select pad 0".to_string(),
                ))?;
        let selector_sink_pad_down =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 1".to_string(),
                ))?;
        let video_scale = gst::ElementFactory::make("videoscale")
            .name("pip_videoscale")
            .build()
            .map_err(InnerError::GlibBool)?;
        let caps = gst::Caps::builder("video/x-raw")
            .field("format", "NV12")
            .field("width", 480)
            .field("height", 270)
            .field("framerate", gst::Fraction::new(30, 1))
            .build();
        let video_scale_caps = gst::ElementFactory::make("capsfilter")
            .property("caps", &caps)
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue = gst::ElementFactory::make("queue")
            .name("pip_queue")
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let sink = gst::ElementFactory::make("autovideosink")
            .name("pip_sink")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(PipSink {
            selector,
            selector_sink_pad_main,
            selector_sink_pad_down,
            video_scale,
            video_convert_caps: video_scale_caps,
            queue,
            sink,
        })
    }
}
