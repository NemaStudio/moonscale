use rocket::{http::Status, post, State};
use rocket_okapi::openapi;

use crate::context::Context;

/// # Create a PlanetScale's compatible database
///
/// This route is used to create a PlanetScale's compatible database.
#[openapi(tag = "database")]
#[post("/database")]
pub fn route_create_database(context: &State<Context>) -> Status {
    Status::Ok
}
