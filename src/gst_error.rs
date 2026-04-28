use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Init: {0}")]
    Init(InnerError),
    #[error("Create Pipeline: {0}")]
    CreatePipeline(InnerError),
    #[error("Link: {0}")]
    LinkStr(String),
    #[error("Link: {0}")]
    Link(InnerError),
    #[error("State change: {0}")]
    StateChange(InnerError),
    #[error("Join")]
    Join,
}

#[derive(Debug, Error)]
pub enum InnerError {
    #[error("gst: {0}")]
    Glib(gstreamer::glib::Error),
    #[error("gst: {0}")]
    GlibBool(gstreamer::glib::BoolError),
    #[error("gst: {0}")]
    Link(gstreamer::PadLinkError),
    #[error("gst: {0}")]
    StateChange(gstreamer::StateChangeError),
    #[error("bus")]
    Bus,
    #[error("request pad: {0}")]
    RequestPad(String),
    #[error("get element")]
    GetElement,
}
