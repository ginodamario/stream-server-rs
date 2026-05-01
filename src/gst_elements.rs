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
    pub(super) tee: gst::Element,
    pub(super) queue_0: gst::Element,
    pub(super) queue_1: gst::Element,
}

impl ElementTrait for MainSrcElements {
    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many([&self.src, &self.caps, &self.queue, &self.tee])
            .map_err(InnerError::GlibBool)?;

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_0.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_1.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            &self.src,
            &self.caps,
            &self.queue,
            &self.tee,
            &self.queue_0,
            &self.queue_1,
        ]
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
        let tee = gst::ElementFactory::make("tee")
            .name("main_tee")
            // .property("allow-not-linked", true)
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_0 = gst::ElementFactory::make("queue")
            .name("main_queue_0")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_1 = gst::ElementFactory::make("queue")
            .name("main_queue_1")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(MainSrcElements {
            src,
            caps,
            queue,
            tee,
            queue_0,
            queue_1,
        })
    }
}

pub(super) struct DownSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) queue: gst::Element,
    pub(super) tee: gst::Element,
    pub(super) queue_0: gst::Element,
    pub(super) queue_1: gst::Element,
}

impl ElementTrait for DownSrcElements {
    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many([&self.src, &self.caps, &self.queue, &self.tee])
            .map_err(InnerError::GlibBool)?;

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_0.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_1.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            &self.src,
            &self.caps,
            &self.queue,
            &self.tee,
            &self.queue_0,
            &self.queue_1,
        ]
    }
}

impl DownSrcElements {
    fn new() -> Result<Self, InnerError> {
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
        let queue = gst::ElementFactory::make("queue")
            .name("down_queue")
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let tee = gst::ElementFactory::make("tee")
            .name("down_tee")
            // .property("allow-not-linked", true)
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_0 = gst::ElementFactory::make("queue")
            .name("down_queue_0")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_1 = gst::ElementFactory::make("queue")
            .name("down_queue_1")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(DownSrcElements {
            src,
            caps,
            queue,
            tee,
            queue_0,
            queue_1,
        })
    }
}

pub(super) struct MainSink {
    pub(super) selector: gst::Element,
    pub(super) selector_sink_pad_0: gst::Pad,
    pub(super) selector_sink_pad_1: gst::Pad,
    pub(super) queue: gst::Element,
    pub(super) sink: gst::Element,
}

impl ElementTrait for MainSink {
    fn get_elements(&self) -> Vec<&gstreamer::Element> {
        vec![&self.selector, &self.queue, &self.sink]
    }
}

impl MainSink {
    fn new() -> Result<Self, InnerError> {
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
        // selector_sink_pad_0.set_property("always-ok", true);
        // selector_sink_pad_1.set_property("always-ok", true);

        Ok(MainSink {
            selector,
            selector_sink_pad_0,
            selector_sink_pad_1,
            queue,
            sink,
        })
    }
}

pub(super) struct PipSink {
    pub(super) selector: gst::Element,
    pub(super) selector_sink_pad_0: gst::Pad,
    pub(super) selector_sink_pad_1: gst::Pad,
    pub(super) video_scale: gst::Element,
    pub(super) video_convert_caps: gst::Element,
    pub(super) queue: gst::Element,
    pub(super) sink: gst::Element,
}

impl ElementTrait for PipSink {
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
    fn new() -> Result<Self, InnerError> {
        let selector = gst::ElementFactory::make("input-selector")
            .name("pip_selector")
            .property("sync-streams", false)
            .build()
            .map_err(InnerError::GlibBool)?;
        let selector_sink_pad_0 =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request pip select pad 0".to_string(),
                ))?;
        let selector_sink_pad_1 =
            selector
                .request_pad_simple("sink_%u")
                .ok_or(InnerError::RequestPad(
                    "Request main select pad 1".to_string(),
                ))?;
        // selector_sink_pad_0.set_property("always-ok", true);
        // selector_sink_pad_1.set_property("always-ok", true);
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
            selector_sink_pad_0,
            selector_sink_pad_1,
            video_scale,
            video_convert_caps: video_scale_caps,
            queue,
            sink,
        })
    }
}

pub(super) struct Elements {
    pub(super) main: MainSrcElements,
    pub(super) down: DownSrcElements,
    pub(super) main_sink: MainSink,
    pub(super) pip_sink: PipSink,
}

impl Elements {
    pub(super) fn new() -> Result<Self, InnerError> {
        let main = MainSrcElements::new()?;
        let down = DownSrcElements::new()?;
        let main_sink = MainSink::new()?;
        let pip_sink = PipSink::new()?;

        Ok(Elements {
            main,
            down,
            main_sink,
            pip_sink,
        })
    }

    pub(super) fn recreate_main(&mut self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.main.remove_from_pipeline(pipeline)?;
        self.main = MainSrcElements::new()?;
        self.main.add_to_pipeline(pipeline)?;
        self.main.link()?;

        Ok(())
    }

    pub(super) fn recreate_down(&mut self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.down.remove_from_pipeline(pipeline)?;
        self.down = DownSrcElements::new()?;
        self.down.add_to_pipeline(pipeline)?;
        self.down.link()?;

        Ok(())
    }

    pub(super) fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.main.add_to_pipeline(pipeline)?;
        self.down.add_to_pipeline(pipeline)?;
        self.main_sink.add_to_pipeline(pipeline)?;
        self.pip_sink.add_to_pipeline(pipeline)?;
        Ok(())
    }

    pub(super) fn link(&self) -> Result<(), InnerError> {
        self.main.link()?;
        self.down.link()?;
        self.main_sink.link()?;
        self.pip_sink.link()?;

        let pad = self.main.queue_0.static_pad("src").unwrap();
        pad.link(&self.main_sink.selector_sink_pad_0)
            .map_err(InnerError::Link)?;

        let pad = self.down.queue_0.static_pad("src").unwrap();
        pad.link(&self.main_sink.selector_sink_pad_1)
            .map_err(InnerError::Link)?;

        let pad = self.main.queue_1.static_pad("src").unwrap();
        pad.link(&self.pip_sink.selector_sink_pad_0)
            .map_err(InnerError::Link)?;

        let pad = self.down.queue_1.static_pad("src").unwrap();
        pad.link(&self.pip_sink.selector_sink_pad_1)
            .map_err(InnerError::Link)?;

        Ok(())
    }
}
