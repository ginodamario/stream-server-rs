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
    pub(super) tee: gst::Element,
    pub(super) queue_0_main_src: gst::Element,
    pub(super) queue_1_pip_src: gst::Element,
}

impl ElementTrait for MainSrcElements {
    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many([&self.src, &self.caps, &self.tee])
            .map_err(InnerError::GlibBool)?;

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_0_main_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_1_pip_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            &self.src,
            &self.caps,
            &self.tee,
            &self.queue_0_main_src,
            &self.queue_1_pip_src,
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
        let tee = gst::ElementFactory::make("tee")
            .name("main_tee")
            // .property("allow-not-linked", true)
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_0_main_src = gst::ElementFactory::make("queue")
            .name("main_queue_0")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_1_pip_src = gst::ElementFactory::make("queue")
            .name("main_queue_1")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(MainSrcElements {
            src,
            caps,
            tee,
            queue_0_main_src,
            queue_1_pip_src,
        })
    }
}

pub(super) struct DownSrcElements {
    pub(super) src: gst::Element,
    pub(super) caps: gst::Element,
    pub(super) tee: gst::Element,
    pub(super) queue_0_main_src: gst::Element,
    pub(super) queue_1_pip_src: gst::Element,
}

impl ElementTrait for DownSrcElements {
    fn link(&self) -> Result<(), InnerError> {
        gst::Element::link_many([&self.src, &self.caps, &self.tee])
            .map_err(InnerError::GlibBool)?;

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_0_main_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        let src_pad = self.tee.request_pad_simple("src_%u").unwrap();
        let sink_pad = self.queue_1_pip_src.static_pad("sink").unwrap();
        src_pad.link(&sink_pad).unwrap();

        Ok(())
    }

    fn get_elements(&self) -> Vec<&gst::Element> {
        vec![
            &self.src,
            &self.caps,
            &self.tee,
            &self.queue_0_main_src,
            &self.queue_1_pip_src,
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
        let tee = gst::ElementFactory::make("tee")
            .name("down_tee")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_0_main_src = gst::ElementFactory::make("queue")
            .name("down_queue_0")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;
        let queue_1_pip_src = gst::ElementFactory::make("queue")
            .name("down_queue_1")
            .property("max-size-buffers", 1u32)
            .property_from_str("leaky", "downstream")
            .build()
            .map_err(InnerError::GlibBool)?;

        Ok(DownSrcElements {
            src,
            caps,
            tee,
            queue_0_main_src,
            queue_1_pip_src,
        })
    }
}

pub(super) struct MainSink {
    pub(super) selector: gst::Element,
    pub(super) selector_sink_pad_main: gst::Pad,
    pub(super) selector_sink_pad_down: gst::Pad,
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

pub(super) struct PipSink {
    pub(super) selector: gst::Element,
    pub(super) selector_sink_pad_main: gst::Pad,
    pub(super) selector_sink_pad_down: gst::Pad,
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
        self.link_main_to_sinks()?;

        Ok(())
    }

    pub(super) fn recreate_down(&mut self, pipeline: &gst::Pipeline) -> Result<(), InnerError> {
        self.down.remove_from_pipeline(pipeline)?;
        self.down = DownSrcElements::new()?;
        self.down.add_to_pipeline(pipeline)?;
        self.down.link()?;
        self.link_down_to_sinks()?;

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

        self.link_main_to_sinks()?;
        self.link_down_to_sinks()?;

        Ok(())
    }

    fn link_main_to_sinks(&self) -> Result<(), InnerError> {
        let pad = self.main.queue_0_main_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            tracing::info!("linking main to main sink");
            pad.link(&self.main_sink.selector_sink_pad_main)
                .map_err(InnerError::Link)?;
        }

        let pad = self.main.queue_1_pip_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            tracing::info!("linking main to pip sink");
            pad.link(&self.pip_sink.selector_sink_pad_main)
                .map_err(InnerError::Link)?;
        }

        Ok(())
    }

    fn link_down_to_sinks(&self) -> Result<(), InnerError> {
        let pad = self.down.queue_0_main_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            pad.link(&self.main_sink.selector_sink_pad_down)
                .map_err(InnerError::Link)?;
        }

        let pad = self.down.queue_1_pip_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            pad.link(&self.pip_sink.selector_sink_pad_down)
                .map_err(InnerError::Link)?;
        }

        Ok(())
    }
}
