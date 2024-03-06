use anyhow::{Context, Result};
use kube::{
    api::{DynamicObject, PatchParams},
    Client, Discovery,
};
use log::error;
use rocket::{http::Status, post, serde::json::Json, State};
use rocket_okapi::openapi;
use serde::Deserialize;
use tera::Tera;

use crate::{kubernetes::kubernetes_apply_document, models::database::CreateDatabaseRequestModel};

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
    kubeclient: &Client,
    discovery: &Discovery,
) -> Result<(), anyhow::Error> {
    let ssapply = PatchParams::apply("kubectl-light").force();
    let mut context: tera::Context = tera::Context::new();

    context.insert("name", variable_data.name.as_str());
    // TODO: Check size formatting
    context.insert("pvc_size", format!("{}Gi", variable_data.size).as_str());
    for doc in multidoc_deserialize(&template_data, &mut context)? {
        let _ = kubernetes_apply_document(kubeclient, discovery, &ssapply, doc).await;
        // let obj: DynamicObject = serde_yaml::from_value(doc)?;
    }
    Ok(())
}

/// # Create a PlanetScale's compatible database
///
/// This route is used to create a PlanetScale's compatible database.
#[openapi(tag = "Database")]
#[post("/database", data = "<request>")]
pub async fn route_create_database(
    context: &State<crate::context::Context>,
    request: Json<CreateDatabaseRequestModel>,
) -> Status {
    let discovery = Discovery::new(context.kubernetes_client.clone())
        .run()
        .await;

    if discovery.is_err() {
        error!("Failed to discover Kubernetes API");
        return Status::InternalServerError;
    }

    let database_creation_result = create_database(
        &context.database_template_yaml_raw,
        &request.0,
        &context.kubernetes_client,
        &discovery.unwrap(),
    )
    .await;

    if database_creation_result.is_err() {
        error!(
            "Failed to create database: {:?}",
            database_creation_result.err()
        );
        return Status::InternalServerError;
    }
    Status::Ok
}
