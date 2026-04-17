use gst::{MessageType, prelude::*};
use gstreamer as gst;
use std::time::{Duration, Instant};

fn main() {
    gst::init().unwrap();

    let source = gst::ElementFactory::make("videotestsrc")
        .name("source")
        .property_from_str("pattern", "smpte")
        .property_from_str("is-live", "true")
        // .property("num-buffers", 200)
        .build()
        .expect("Could not create source element.");
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", "NV12")
        .field("width", 1920)
        .field("height", 1080)
        .field("framerate", gst::Fraction::new(30, 1))
        .build();
    let capsfilter = gst::ElementFactory::make("capsfilter")
        .property("caps", &caps)
        .build()
        .expect("Could not create caps element.");
    let queue = gst::ElementFactory::make("queue")
        .name("queue")
        .build()
        .expect("Could not create queue element.");
    let watchdog = gst::ElementFactory::make("watchdog")
        .name("watchdog")
        .build()
        .expect("Could not create watchdog element");

    let source_ball = gst::ElementFactory::make("videotestsrc")
        .name("source-ball")
        .property_from_str("pattern", "ball")
        .property_from_str("is-live", "true")
        .build()
        .expect("Could not create source element.");
    let caps_ball = gst::Caps::builder("video/x-raw")
        .field("format", "I420")
        .field("width", 1280)
        .field("height", 720)
        .field("framerate", gst::Fraction::new(10, 1))
        .build();
    let capsfilter_ball = gst::ElementFactory::make("capsfilter")
        .property("caps", &caps_ball)
        .build()
        .expect("Could not create caps element.");
    let queue_ball = gst::ElementFactory::make("queue")
        .name("queue-ball")
        .build()
        .expect("Could not create queue element.");

    let inputsel = gst::ElementFactory::make("input-selector")
        .name("inputsel")
        .build()
        .expect("Could not create input-selector element.");
    let sink = gst::ElementFactory::make("autovideosink")
        .name("sink")
        .build()
        .expect("Could not create sink element");

    let pipeline = gst::Pipeline::with_name("test-pipeline");

    pipeline
        .add_many([
            &source,
            &capsfilter,
            &queue,
            &watchdog,
            &source_ball,
            &capsfilter_ball,
            &queue_ball,
            &inputsel,
            &sink,
        ])
        .unwrap();

    gst::Element::link_many([&source, &capsfilter, &queue, &watchdog, &inputsel])
        .expect("Elements could not be linked");
    gst::Element::link_many([&source_ball, &capsfilter_ball, &queue_ball, &inputsel])
        .expect("Elements could not be linked");
    gst::Element::link_many([&inputsel, &sink]).expect("Elements for sink could not be linked");
    // source.link(&sink).expect("Elements could not be linked.");

    println!("sel src_pads: {:?}", inputsel.src_pads());
    println!("sel sink_pads: {:?}", inputsel.sink_pads());
    for pad in &inputsel.sink_pads() {
        println!("sink name: {}", pad.name());
    }

    // let first_pad = &inputsel.sink_pads()[1];
    let sink_pads = inputsel.sink_pads();
    let first_pad = sink_pads.first().unwrap();
    let second_pad = sink_pads.get(1).unwrap();

    inputsel.set_property("active-pad", first_pad);

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to se the pipeline to the 'Playing' state");

    let mut instant = Instant::now();
    let mut sel = false;
    let bus = pipeline.bus().unwrap();
    while (true) {
        let msg = bus.timed_pop_filtered(
            gst::ClockTime::from_mseconds(100),
            &[MessageType::Error, MessageType::Eos],
        );

        use gst::MessageView;
        match msg {
            Some(msg) => match msg.view() {
                MessageView::Error(err) => {
                    if let Some(obj) = err.src()
                        && let Some(element) = obj.downcast_ref::<gst::Element>()
                        && element.has_as_ancestor(&watchdog)
                    {
                        println!("Watchdog Error");
                        // Don't break.
                    } else {
                        eprintln!(
                            "Error recieved from element {:?}: {}",
                            err.src().map(|s| s.path_string()),
                            err.error()
                        );
                        eprintln!("Debugging information: {:?}", err.debug());
                        break;
                    }
                }
                MessageView::Eos(_) => {
                    println!("eos");
                }
                _ => {}
            },
            None => {
                if instant.elapsed() > Duration::from_secs(2) {
                    if sel {
                        println!("sel true");
                        // if !first_pad.is_blocking() {
                        //     inputsel.set_property("active-pad", first_pad);
                        //     sel = !sel;
                        // } else {
                        //     println!("first pad not active");
                        // }
                        // pipeline
                        //     .set_state(gst::State::Playing)
                        //     .expect("Unable to se the pipeline to the 'Playing' state");
                        source
                            .set_state(gst::State::Playing)
                            .expect("Unable to set source state to playing.");
                    } else {
                        println!("sel false");
                        source
                            .set_state(gst::State::Null)
                            .expect("Unable to set source state to null.");
                        // if !second_pad.is_blocking() {
                        //     inputsel.set_property("active-pad", second_pad);
                        //     sel = !sel;
                        // } else {
                        //     println!("second pad not active");
                        // }
                    }
                    sel = !sel;
                    instant = Instant::now();
                }
            }
        }
    }
    // for msg in bus.iter_timed_filtered(
    //     gst::ClockTime::NONE,
    //     &[MessageType::Error, MessageType::Eos],
    // ) {
    //     use gst::MessageView;
    //
    //     match msg.view() {
    //         MessageView::Error(err) => {
    //             eprintln!(
    //                 "Error recieved from element {:?}: {}",
    //                 err.src().map(|s| s.path_string()),
    //                 err.error()
    //             );
    //             eprintln!("Debugging information: {:?}", err.debug());
    //             break;
    //         }
    //         MessageView::Eos(_) => break,
    //         _ => {
    //             println!("noop");
    //         }
    //     }
    // }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipline to the 'Null' state");

    println!("Hello, world!");
}
