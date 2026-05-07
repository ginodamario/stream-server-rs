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

    fn set_state(&self, state: gst::State) -> Result<(), InnerError>;

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
