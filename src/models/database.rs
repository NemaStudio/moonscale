use std::time;

use k8s_openapi::chrono::DateTime;
use rocket::time::serde::rfc3339;
use rocket::time::{OffsetDateTime, Time};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatabaseRequestModel {
    // TODO: Set constraints for the name
    /// Required: The name of the database, usually just the ID of the pull requested
    /// associated to this database
    pub name: String,

    /// Required: The size of the database in GB, this will always be clamped between
    /// 1GB and 5GB
    pub size: usize,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatabaseResponseModel {
    /// The URL to the PlanetScale API pointing to the new underlying database.
    pub planetscale_api_url: String,
    // A timestamp corresponding to the approximate time where the database will be deleted.
    //pub deletion_timestamp: OffsetDateTime,
}

pub struct Database {}
