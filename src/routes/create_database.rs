use anyhow::{Context, Result};
use k8s_openapi::{api::discovery, serde_json};
use kube::{
    api::{ApiResource, DynamicObject, GroupVersionKind, Patch, PatchParams},
    core::gvk,
    discovery::{ApiCapabilities, Scope},
    Api, Client, Discovery, ResourceExt,
};
use log::error;
use rocket::{
    http::{ContentType, Status},
    post, State,
};
use rocket_okapi::openapi;
use serde::Deserialize;
use tera::Tera;

fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: Client,
    ns: Option<&str>,
    all: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}

fn multidoc_deserialize(data: &str) -> Result<Vec<serde_yaml::Value>, ()> {
    let mut docs = vec![];
    let mut context = tera::Context::new();
    let mut tera = Tera::default();

    tera.add_raw_template("template", data);
    context.insert("name", "pr-108");
    let render_result = tera.render("template", &context);

    if render_result.is_err() {
        error!("Failed to render template: {:?}", render_result);
        return Err(());
    }
    for de in serde_yaml::Deserializer::from_str(render_result.unwrap().as_str()) {
        let dedoc = serde_yaml::Value::deserialize(de);

        if dedoc.is_err() {
            error!("Failed to deserialize yaml: {:?}", dedoc);
            return Err(());
        }
        docs.push(dedoc.unwrap());
    }
    Ok(docs)
}

async fn create_database(data: &str, kubeclient: &Client, discovery: &Discovery) -> Result<(), ()> {
    let formatted_template = multidoc_deserialize(data);
    let ssapply = PatchParams::apply("kubectl-light").force();

    if formatted_template.is_err() {
        error!("Failed to deserialize yaml: {:?}", formatted_template);
        return Err(());
    }
    for doc in multidoc_deserialize(&data)? {
        // TODO: Refactor
        let obj: DynamicObject = serde_yaml::from_value(doc).unwrap_or_else(|err| {
            error!(
                "Failed to interpret yaml as a valid Kubernetes object: {}",
                err
            );
            std::process::exit(1);
        });
        let namespace = obj.metadata.namespace.as_deref().or(Some("moonscale"));
        let gvk = if let Some(tm) = &obj.types {
            GroupVersionKind::try_from(tm)
        } else {
            panic!("No type metadata in {:?}", obj);
        };
        let name = obj.name_any();

        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk.unwrap()) {
            let api = dynamic_api(ar, caps, kubeclient.clone(), namespace, false);
            let data: serde_json::Value = serde_json::to_value(&obj).unwrap();
            let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await;

            if _r.is_err() {
                error!("Failed to apply document for {:?}: {:?}", name, _r);
            } else {
                println!("Applied {:?}", &obj.types.unwrap());
            }
        } else {
            error!(
                "Cannot apply document for unknown {:?}",
                &obj.types.unwrap()
            );
        }
    }
    Ok(())
}

/// # Create a PlanetScale's compatible database
///
/// This route is used to create a PlanetScale's compatible database.
#[openapi(tag = "database")]
#[post("/database")]
pub async fn route_create_database(context: &State<crate::context::Context>) -> Status {
    let discovery = Discovery::new(context.kubernetes_client.clone())
        .run()
        .await;

    if discovery.is_err() {
        error!("Failed to discover Kubernetes API");
        return Status::InternalServerError;
    }

    let rendered_template = create_database(
        &context.database_template_yaml_raw,
        &context.kubernetes_client,
        &discovery.unwrap(),
    )
    .await;

    if rendered_template.is_err() {
        return Status::InternalServerError;
    }
    Status::Ok
}
