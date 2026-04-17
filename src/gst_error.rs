use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Init: {0}")]
    Init(InnerError),
    #[error("Creating Pipeline: {0}")]
    CreatingPipeline(InnerError),
    #[error("Join")]
    Join,
}

#[derive(Debug, Error)]
pub enum InnerError {
    #[error("gst: {0}")]
    GlibError(gstreamer::glib::Error),
    #[error("gst: {0}")]
    GlibBoolError(gstreamer::glib::BoolError),
}


