use gst::prelude::*;
use gstreamer as gst;
use tracing_subscriber::layer::Identity;

use crate::gst_elements::ElementTrait;
use crate::gst_error::InnerError;

pub(crate) struct MainSrcElements {
    pub(crate) src: gst::Element,
    pub(crate) caps: gst::Element,
    pub(crate) identity: gst::Element,
    pub(crate) tee: gst::Element,
    pub(crate) queue_main_src: gst::Element,
    pub(crate) queue_pip_src: gst::Element,
    pub(crate) queue_save_src: gst::Element,
}

impl ElementTrait for MainSrcElements {
    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        Ok(())
    }

    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many([&self.src, &self.caps, &self.identity, &self.tee])
            .map_err(InnerError::GlibBool)?;

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_main_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_pip_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_save_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            &self.src,
            &self.caps,
            &self.identity,
            &self.tee,
            &self.queue_main_src,
            &self.queue_pip_src,
            &self.queue_save_src,
        ]
    }
}

impl MainSrcElements {
    pub(crate) fn new() -> Result<Self, InnerError> {
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
        let identity = gst::ElementFactory::make("identity")
            .name("main_id")
            // .property("eos-after", 60)
            .property("error-after", 60)
            .build()
            .map_err(InnerError::GlibBool)?;
        let tee = gst::ElementFactory::make("tee")
            .name("main_tee")
            .property("allow-not-linked", true)
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_main_src = gst::ElementFactory::make("queue")
            .name("main_queue_0")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_pip_src = gst::ElementFactory::make("queue")
            .name("main_queue_1")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_save_src = gst::ElementFactory::make("queue")
            .name("main_queue_2")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(MainSrcElements {
            src,
            caps,
            identity,
            tee,
            queue_main_src,
            queue_pip_src,
            queue_save_src,
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

    pub(crate) fn get_save_src_pad(&self) -> Result<gst::Pad, InnerError> {
        let pad = self.queue_save_src.static_pad("src").unwrap();
        Ok(pad)
    }
}
