use gst::prelude::*;
use gstreamer as gst;

use crate::gst_elements::ElementTrait;
use crate::gst_error::InnerError;

pub(crate) struct DownSrcElements {
    pub(crate) src: gst::Element,
    pub(crate) caps: gst::Element,
    pub(crate) tee: gst::Element,
    pub(crate) queue_main_src: gst::Element,
    pub(crate) queue_pip_src: gst::Element,
}

impl ElementTrait for DownSrcElements {
    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        Ok(())
    }

    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many([&self.src, &self.caps, &self.tee])
            .map_err(InnerError::GlibBool)?;

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_main_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_pip_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            &self.src,
            &self.caps,
            &self.tee,
            &self.queue_main_src,
            &self.queue_pip_src,
        ]
    }
}

impl DownSrcElements {
    pub(crate) fn new() -> Result<Self, InnerError> {
        let src = gst::ElementFactory::make("videotestsrc")
            .name("down_src")
            .property_from_str("pattern", "ball")
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
            .expect("Could not create caps element.");
        let tee = gst::ElementFactory::make("tee")
            .name("down_tee")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_main_src = gst::ElementFactory::make("queue")
            .name("down_queue_0")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_pip_src = gst::ElementFactory::make("queue")
            .name("down_queue_1")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(DownSrcElements {
            src,
            caps,
            tee,
            queue_main_src,
            queue_pip_src,
        })
    }

    pub(crate) fn get_main_src_pad(&self) -> Result<gst::Pad, InnerError> {
        let pad = self.queue_main_src.static_pad("src").unwrap();
        Ok(pad)
    }

    pub(crate) fn get_pip_src_pad(&self) -> Result<gst::Pad, InnerError> {
        let pad = self.queue_pip_src.static_pad("src").unwrap();
        Ok(pad)
    }
}
