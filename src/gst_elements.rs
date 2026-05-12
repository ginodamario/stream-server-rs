use gst::prelude::*;
use gstreamer as gst;

pub(crate) use gst_element_trait::ElementTrait;

use crate::gst_error::InnerError;
use gst_down_src::DownSrcElements;
use gst_main_save::MainSaveElements;
use gst_main_sink::MainSink;
use gst_main_src::MainSrcElements;
use gst_pip_sink::PipSink;

mod gst_down_src;
mod gst_element_trait;
mod gst_main_save;
mod gst_main_sink;
mod gst_main_src;
mod gst_pip_sink;

pub(super) struct Elements {
    pub(super) main: MainSrcElements,
    pub(super) down: DownSrcElements,
    pub(super) main_sink: MainSink,
    pub(super) pip_sink: PipSink,
    pub(super) main_save: MainSaveElements,
}

impl Elements {
    pub(super) fn new() -> Result<Self, InnerError> {
        let main = MainSrcElements::new()?;
        let down = DownSrcElements::new()?;
        let main_sink = MainSink::new()?;
        let pip_sink = PipSink::new()?;
        let main_save = MainSaveElements::new()?;

        Ok(Elements {
            main,
            down,
            main_sink,
            pip_sink,
            main_save,
        })
    }

    pub(super) fn set_state(&mut self, state: gst::State) {
        self.main.set_state(state).unwrap();
        self.down.set_state(state).unwrap();
        self.main_sink.set_state(state).unwrap();
        self.pip_sink.set_state(state).unwrap();
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
        // self.main_save.add_to_pipeline(pipeline)?;
        Ok(())
    }

    pub(super) fn link(&self) -> Result<(), InnerError> {
        self.main.link()?;
        self.down.link()?;
        self.main_sink.link()?;
        self.pip_sink.link()?;
        self.main_save.link()?;

        self.link_main_to_sinks()?;
        self.link_down_to_sinks()?;

        Ok(())
    }

    pub(super) fn link_main_to_save(&self) -> Result<(), InnerError> {
        // Link MainSrc to MainSave.
        let src_pad = self.main.get_save_src_pad().unwrap();
        let sink_pad = self.main_save.enc.static_pad("sink").unwrap();
        if !src_pad.is_linked() {
            src_pad.link(&sink_pad).unwrap();
        }

        Ok(())
    }

    fn link_main_to_sinks(&self) -> Result<(), InnerError> {
        let pad = self.main.queue_main_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            tracing::info!("linking main to main sink");
            pad.link(&self.main_sink.selector_sink_pad_main)
                .map_err(InnerError::Link)?;
        }

        let pad = self.main.queue_pip_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            tracing::info!("linking main to pip sink");
            pad.link(&self.pip_sink.selector_sink_pad_main)
                .map_err(InnerError::Link)?;
        }

        Ok(())
    }

    fn link_down_to_sinks(&self) -> Result<(), InnerError> {
        let pad = self.down.queue_main_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            pad.link(&self.main_sink.selector_sink_pad_down)
                .map_err(InnerError::Link)?;
        }

        let pad = self.down.queue_pip_src.static_pad("src").unwrap();
        if !pad.is_linked() {
            pad.link(&self.pip_sink.selector_sink_pad_down)
                .map_err(InnerError::Link)?;
        }

        Ok(())
    }
}
