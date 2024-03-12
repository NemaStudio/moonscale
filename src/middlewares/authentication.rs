use okapi::openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    Request,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

use crate::context::Context;

pub struct ApiKey(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let context = request.rocket().state::<Context>().unwrap();

        match request.headers().get_one("Authorization") {
            None => Outcome::Error((Status::BadRequest, ())),
            Some(key) if key == format!("Bearer {}", context.config.api_key) => {
                Outcome::Success(ApiKey(key.to_owned()))
            }
            Some(_) => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for ApiKey {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let security_scheme = SecurityScheme {
            description: Some("Requires a Bearer token to access.".to_owned()),
            data: SecuritySchemeData::Http {
                scheme: "bearer".to_owned(),
                bearer_format: Some("bearer".to_owned()),
            },
            extensions: Object::default(),
        };

        let mut security_req = SecurityRequirement::new();

        security_req.insert("HttpAuth".to_owned(), Vec::new());
        Ok(RequestHeaderInput::Security(
            "HttpAuth".to_owned(),
            security_scheme,
            security_req,
        ))
    }
}
