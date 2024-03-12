use crate::middlewares::authentication::ApiKey;
use crate::models::database::CreateDatabaseResponseModel;
use crate::{kubernetes::kubernetes_apply_document, models::database::CreateDatabaseRequestModel};
use anyhow::{Context, Result};
use base64::prelude::*;
use kube::{api::PatchParams, Discovery};
use log::error;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rocket::response::status::{self};
use rocket::{http::Status, post, serde::json::Json, State};
use rocket_okapi::openapi;
use serde::Deserialize;
use tera::Tera;

fn multidoc_deserialize(
    data: &str,
    context: &mut tera::Context,
) -> Result<Vec<serde_yaml::Value>, anyhow::Error> {
    let mut docs = vec![];
    let mut tera = Tera::default();

    tera.add_raw_template("template", data)
        .context("Rendering error")?;
    let render_result = tera
        .render("template", context)
        .context("Failed to render template, check your yaml file.")?;

    for de in serde_yaml::Deserializer::from_str(render_result.as_str()) {
        let dedoc = serde_yaml::Value::deserialize(de)
            .context("Couldn't deserialize yaml. Check format.")?;

        docs.push(dedoc);
    }
    Ok(docs)
}

async fn create_database(
    template_data: &str,
    variable_data: &CreateDatabaseRequestModel,
    context: &crate::context::Context,
    discovery: &Discovery,
) -> Result<CreateDatabaseResponseModel, anyhow::Error> {
    let ssapply = PatchParams::apply("kubectl-light").force();
    let mut template_context: tera::Context = tera::Context::new();
    let random_password = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    template_context.insert("name", variable_data.name.as_str());
    template_context.insert("domain", context.config.ingress_domain.as_str());
    template_context.insert("resource_ttl", &context.config.resource_ttl);
    // TODO: This will be a problem if you create a database that already exists, as
    // the password will be different from what mysql expects
    // and also because the pod isn't restarted when the secret is updated
    template_context.insert(
        "root_password",
        &BASE64_STANDARD.encode(random_password.as_str()),
    );
    // TODO: Check size formatting
    template_context.insert(
        "pvc_size",
        format!("{}Gi", num::clamp(variable_data.size, 1, 5)).as_str(),
    );
    for doc in multidoc_deserialize(&template_data, &mut template_context)? {
        let _ =
            kubernetes_apply_document(&context.kubernetes_client, discovery, &ssapply, doc).await;
        // let obj: DynamicObject = serde_yaml::from_value(doc)?;
    }

    Ok(CreateDatabaseResponseModel {
        database_name: variable_data.name.clone(),
        database_password: random_password,
        database_username: "root".to_owned(),
        planetscale_api_url: format!(
            "https://moonscale-instance-{}.{}",
            variable_data.name, context.config.ingress_domain
        )
        .to_string(),
    })
}

/// # Create a PlanetScale's compatible database
///
/// This route is used to create a PlanetScale's compatible database.
#[openapi(tag = "Database")]
#[post("/database", data = "<request>")]
pub async fn route_create_database(
    context: &State<crate::context::Context>,
    request: Json<CreateDatabaseRequestModel>,
    _key: ApiKey,
) -> Result<status::Custom<Json<CreateDatabaseResponseModel>>, Status> {
    let discovery = Discovery::new(context.kubernetes_client.clone())
        .run()
        .await;

    if discovery.is_err() {
        error!("Failed to discover Kubernetes API");
        return Err(Status::InternalServerError);
    }

    let database_creation_result = create_database(
        &context.database_template_yaml_raw,
        &request.0,
        &context,
        &discovery.unwrap(),
    )
    .await;

    if database_creation_result.is_err() {
        error!(
            "Failed to create database: {:?}",
            database_creation_result.err()
        );
        return Err(Status::InternalServerError);
    }

    Ok(status::Custom(
        Status::Created,
        Json(database_creation_result.unwrap()),
    ))
}
