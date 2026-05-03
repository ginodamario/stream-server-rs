use gst::{MessageType, prelude::*};
use gstreamer as gst;

use crate::gst_elements::{ElementTrait, Elements};
use crate::gst_error::{Error, InnerError};
use crate::gst_probe::GstProbe;
use crate::gst_source::Source;

pub(crate) struct Pipeline {
    pipeline: gst::Pipeline,
    elements: Elements,
    bus: gst::Bus,
    main_probe: GstProbe,
    down_probe: GstProbe,
}

impl Pipeline {
    pub(crate) fn new() -> Result<Self, InnerError> {
        let pipeline = gst::Pipeline::with_name("pipeline");
        let elements = Elements::new().unwrap();

        elements.add_to_pipeline(&pipeline).unwrap();
        elements.link().unwrap();

        let bus = pipeline.bus().unwrap();

        let main_probe = GstProbe::new(&elements.main.src);
        let down_probe = GstProbe::new(&elements.down.src);

        Ok(Self {
            pipeline,
            elements,
            bus,
            main_probe,
            down_probe,
        })
    }

    pub(crate) fn run_loop<F>(&mut self, mut call: F)
    where
        F: FnMut(&mut Self) -> bool,
    {
        while (true) {
            let msg = self.bus.timed_pop_filtered(
                gst::ClockTime::from_mseconds(100),
                &[MessageType::Error, MessageType::Eos],
            );

            use gst::MessageView;
            match msg {
                Some(msg) => match msg.view() {
                    MessageView::Error(err) => {
                        tracing::error!(
                            "Error recieved from element {:?}: {}",
                            err.src().map(|s| s.path_string()),
                            err.error()
                        );
                        tracing::error!("Debugging information: {:?}", err.debug());
                        break;
                    }
                    MessageView::Eos(_) => {
                        tracing::info!("eos");
                    }
                    _ => {}
                },
                None => {
                    if self.main_probe.is_stale() {
                        Self::teardown_main(&mut self.elements).unwrap();
                        self.main_probe.stop();
                    }
                    if self.down_probe.is_stale() {
                        Self::teardown_down(&mut self.elements).unwrap();
                        self.down_probe.stop();
                    }

                    if !call(self) {
                        break;
                    }
                }
            }
        }
    }

    pub(crate) fn set_state(&mut self, state: gst::State) {
        self.pipeline.set_state(state).unwrap();
        self.elements.set_state(state);
    }

    pub(crate) fn simulate_main_stop(&self) {
        self.elements.main.src.set_state(gst::State::Null).unwrap();
    }

    pub(crate) fn simulate_down_stop(&self) {
        self.elements.down.src.set_state(gst::State::Null).unwrap();
    }

    pub(crate) fn set_main_state(&mut self, state: gst::State) {
        self.elements.main.set_state(state).unwrap();
    }

    pub(crate) fn set_down_state(&mut self, state: gst::State) {
        self.elements.down.set_state(state).unwrap();
    }

    pub(crate) fn switch_main_sink(&self, source: Source) {
        let pad = match source {
            Source::Main => &self.elements.main_sink.selector_sink_pad_main,
            Source::Down => &self.elements.main_sink.selector_sink_pad_down,
        };
        self.elements
            .main_sink
            .selector
            .set_property("active-pad", pad);
    }

    pub(crate) fn switch_pip_sink(&self, source: Source) {
        let pad = match source {
            Source::Main => &self.elements.pip_sink.selector_sink_pad_main,
            Source::Down => &self.elements.pip_sink.selector_sink_pad_down,
        };
        self.elements
            .pip_sink
            .selector
            .set_property("active-pad", pad);
    }

    pub(crate) fn recreate_main(&mut self) {
        self.elements.recreate_main(&self.pipeline).unwrap();
        self.main_probe = GstProbe::new(&self.elements.main.src);
    }

    pub(crate) fn recreate_down(&mut self) {
        self.elements.recreate_down(&self.pipeline).unwrap();
        self.down_probe = GstProbe::new(&self.elements.down.src);
    }

    fn teardown_main(elements: &mut Elements) -> Result<(), InnerError> {
        tracing::info!("teardown main");

        // Stop all main elements.
        elements.main.set_state(gst::State::Null).unwrap();

        let main_src_pad = elements.main.queue_main_src.static_pad("src").unwrap();
        let pip_src_pad = elements.main.queue_pip_src.static_pad("src").unwrap();

        if main_src_pad.is_linked() {
            tracing::info!("unlink main to main_sink");
            let selected_pad: gst::Pad = elements.main_sink.selector.property("active-pad");
            if selected_pad == elements.main_sink.selector_sink_pad_main {
                elements
                    .main
                    .queue_main_src
                    .unlink(&elements.main_sink.selector);
            }
        }

        if pip_src_pad.is_linked() {
            tracing::info!("unlink main to pip_sink");
            let selected_pad: gst::Pad = elements.pip_sink.selector.property("active-pad");
            if selected_pad == elements.pip_sink.selector_sink_pad_main {
                elements
                    .main
                    .queue_pip_src
                    .unlink(&elements.pip_sink.selector);
            }
        }

        Ok(())
    }

    fn teardown_down(elements: &mut Elements) -> Result<(), InnerError> {
        tracing::info!("teardown down");

        // Stop all main elements.
        elements.down.set_state(gst::State::Null).unwrap();

        let main_src_pad = elements.down.queue_main_src.static_pad("src").unwrap();
        let pip_src_pad = elements.down.queue_pip_src.static_pad("src").unwrap();

        if main_src_pad.is_linked() {
            tracing::info!("unlink down to main_sink");
            let selected_pad: gst::Pad = elements.main_sink.selector.property("active-pad");
            if selected_pad == elements.main_sink.selector_sink_pad_down {
                elements
                    .down
                    .queue_main_src
                    .unlink(&elements.main_sink.selector);
            }
        }

        if pip_src_pad.is_linked() {
            tracing::info!("unlink down to pip_sink");
            let selected_pad: gst::Pad = elements.pip_sink.selector.property("active-pad");
            if selected_pad == elements.pip_sink.selector_sink_pad_down {
                elements
                    .down
                    .queue_pip_src
                    .unlink(&elements.pip_sink.selector);
            }
        }

        Ok(())
    }
}
