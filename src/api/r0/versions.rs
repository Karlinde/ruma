//! Endpoints for information about supported versions of the Matrix spec.

use iron::{status, Handler, IronResult, Request, Response};

use crate::modifier::SerializableResponse;

/// The /versions endpoint.
#[derive(Debug, Serialize)]
pub struct Versions {
    versions: Vec<&'static str>,
}

impl Versions {
    /// Returns the list of supported `Versions` of the Matrix spec.
    pub fn supported() -> Self {
        Versions {
            versions: vec!["r0.2.0"],
        }
    }
}

impl Handler for Versions {
    fn handle(&self, _request: &mut Request<'_, '_>) -> IronResult<Response> {
        Ok(Response::with((status::Ok, SerializableResponse(&self))))
    }
}
