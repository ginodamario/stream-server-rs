use gst::prelude::*;
use gstreamer as gst;

use crate::gst_error::InnerError;

pub(super) trait ElementTrait {
    fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .add_many(self.get_elements())
            .map_err(InnerError::GlibBool)
    }

    fn remove_from_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        pipeline
            .remove_many(self.get_elements())
            .map_err(InnerError::GlibBool)
    }

    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many(self.get_elements()).map_err(InnerError::GlibBool)?;

        Ok(())
    }

    fn get_last(&self) -> Result<&gst::Element, InnerError> {
        let e = *self.get_elements().last().ok_or(InnerError::GetElement)?;
        Ok(e)
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

impl MainSrcElements {
    fn new() -> Result<Self, InnerError> {
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
        let queue = gst::ElementFactory::make("queue")
            .name("main_queue")
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(MainSrcElements { src, caps, queue })
    }
}

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

impl DownSrcElements {
    fn new() -> Result<Self, InnerError> {
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
            .property_from_str("leaky", "downstream")
            .build()
            .expect("Could not create queue element.");

        Ok(DownSrcElements { src, caps, queue })
    }
}

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

impl Sink {
    fn new() -> Result<Self, InnerError> {
        let selector = gst::ElementFactory::make("input-selector")
            .name("selector")
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

        let selector_sink_pad_0 =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 0".to_string(),
                ))?;

        let selector_sink_pad_1 =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 1".to_string(),
                ))?;

        Ok(Sink {
            selector,
            selector_sink_pad_0,
            selector_sink_pad_1,
            queue,
            sink,
        })
    }
}

pub(super) struct Elements {
    pub(super) main: MainSrcElements,
    pub(super) down: DownSrcElements,
    pub(super) main_sink: Sink,
}

impl Elements {
    pub(super) fn new() -> Result<Self, InnerError> {
        let main = MainSrcElements::new()?;
        let down = DownSrcElements::new()?;
        let sink = Sink::new()?;

        Ok(Elements {
            main,
            down,
            main_sink: sink,
        })
    }

    pub(super) fn recreate_main(&mut self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.main.remove_from_pipeline(pipeline)?;
        self.main = MainSrcElements::new()?;
        self.main.add_to_pipeline(pipeline)?;
        self.main.link()?;
        self.link_element_to_sink_pad(self.main.get_last()?, &self.main_sink.selector_sink_pad_0)?;

        Ok(())
    }

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

        self.link_element_to_sink_pad(self.main.get_last()?, &self.main_sink.selector_sink_pad_0)?;
        self.link_element_to_sink_pad(self.down.get_last()?, &self.main_sink.selector_sink_pad_1)?;

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
