#![allow(unused)]

use gst::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use gstreamer as gst;

enum State {
    Stop,
    Running,
}

pub(crate) struct GstProbe {
    cnt: Arc<AtomicUsize>,
    prev_cnt: usize,
    pad_probe_id: gst::PadProbeId,
    state: State,
}

impl GstProbe {
    pub(crate) fn new(element: &gst::Element) -> Self {
        let pad = element.static_pad("src").unwrap();
        let cnt = Arc::new(AtomicUsize::new(0));
        let cnt_clone = cnt.clone();

        let pad_probe_id = pad
            .add_probe(gst::PadProbeType::BUFFER, move |_pad, _info| {
                cnt_clone.fetch_add(1, Ordering::Relaxed);
                gst::PadProbeReturn::Ok
            })
            .unwrap();

        Self {
            cnt,
            prev_cnt: 0,
            pad_probe_id,
            state: State::Running,
        }
    }

    pub(crate) fn stop(&mut self) {
        self.state = State::Stop;
    }

    pub(crate) fn is_stale(&mut self) -> bool {
        let cnt = self.cnt.load(Ordering::Relaxed);
        let is_stale = cnt == self.prev_cnt;
        self.prev_cnt = cnt;

        match self.state {
            State::Stop => false,
            State::Running => is_stale,
        }
    }
}
