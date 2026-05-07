use gst::prelude::*;
use gstreamer as gst;

use crate::gst_error::InnerError;
use crate::gst_element_trait::ElementTrait;

pub(crate) struct MainSaveElements {
    // pub(crate) valve: gst::Element,
    pub(crate) enc: gst::Element,
    pub(crate) parse: gst::Element,
    pub(crate) mux: gst::Element,
    pub(crate) queue: gst::Element,
    pub(crate) sink: gst::Element,
}

impl ElementTrait for MainSaveElements {
    fn set_state(&self, state: gst::State) -> Result<(), InnerError> {
        for element in self.get_elements() {
            element.set_state(state).map_err(InnerError::StateChange)?;
        }

        // match state {
        //     gst::State::VoidPending | gst::State::Null | gst::State::Ready | gst::State::Paused => {
        //         self.valve.set_property("drop", true)
        //     }
        //     gst::State::Playing => self.valve.set_property("drop", false),
        // }

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            // &self.valve,
            &self.enc,
            &self.parse,
            &self.mux,
            &self.queue,
            &self.sink,
        ]
    }
}

impl MainSaveElements {
    pub(crate) fn new() -> Result<Self, InnerError> {
        // let valve = gst::ElementFactory::make("valve")
        //     .name("main_save_valve")
        //     .property("drop", true)
        //     .build()
        //     .map_err(InnerError::GlibBool)?;
        let enc = gst::ElementFactory::make("vah265enc")
            .name("main_save_enc")
            .property("bitrate", 2048u32)
            .build()
            .map_err(InnerError::GlibBool)?;
        let parse = gst::ElementFactory::make("h265parse")
            .name("main_save_parse")
            .build()
            .map_err(InnerError::GlibBool)?;
        let mux = gst::ElementFactory::make("matroskamux")
            .name("main_save_mux")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue = gst::ElementFactory::make("queue")
            .property("max-size-buffers", 300u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let sink = gst::ElementFactory::make("filesink")
            .property_from_str("location", "main.mkv")
            .property("async", false)
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(MainSaveElements {
            // valve,
            enc,
            parse,
            mux,
            queue,
            sink,
        })
    }
}


