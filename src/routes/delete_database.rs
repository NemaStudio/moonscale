use crate::{
    kubernetes::get_database_password,
    middlewares::authentication::ApiKey,
    models::database::{DatabaseInstanceModel, ListDatabaseResponseModel},
};
use k8s_openapi::api::apps::v1::StatefulSet;
use kube::{
    api::{DynamicObject, ListParams},
    discovery, Api,
};
use log::{debug, info};
use rocket::{delete, get, http::Status, response::status, serde::json::Json, State};
use rocket_okapi::openapi;

/// # Delete a managed database
///
/// This route is used to delete a deployed moonscale database.
#[openapi(tag = "Database")]
#[delete("/database/<instance>")]
pub async fn route_delete_database(
    instance: String,
    context: &State<crate::context::Context>,
    _key: ApiKey,
) -> Status {
    let apigroup = discovery::group(&context.kubernetes_client, "kube.rs")
        .await
        .unwrap();
    let (ar, caps) = apigroup.recommended_kind("Document").unwrap();
    let resources_gen_api: Api<DynamicObject> =
        Api::namespaced_with(context.kubernetes_client.clone(), "moonscale", &ar);
    let resources = resources_gen_api
        .list(
            &ListParams::default().labels(
                format!(
                    "app.kubernetes.io/managed-by=Moonscale,app.kubernetes.io/instance={}",
                    instance
                )
                .as_str(),
            ),
        )
        .await;

    for resource in resources.unwrap() {
        // let resource = resource.unwrap();
        // let resource_name = resource.metadata.name.as_ref().unwrap();
        // let _ = resources_gen_api.delete(resource_name, &Default::default()).await;
        debug!(
            "Deleted resource: {}",
            resource.metadata.name.unwrap().as_str()
        );
    }
    Status::Ok
}
