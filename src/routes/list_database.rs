use crate::{
    kubernetes::get_database_password,
    middlewares::authentication::ApiKey,
    models::database::{DatabaseInstanceModel, ListDatabaseResponseModel},
};
use k8s_openapi::api::apps::v1::StatefulSet;
use kube::{api::ListParams, Api};
use log::{debug, info};
use rocket::{get, http::Status, response::status, serde::json::Json, State};
use rocket_okapi::openapi;

/// # List all managed databases
///
/// This route is used to list deployed moonscale databases.
#[openapi(tag = "Database")]
#[get("/database")]
pub async fn route_list_database(
    context: &State<crate::context::Context>,
    _key: ApiKey,
) -> Result<status::Custom<Json<ListDatabaseResponseModel>>, Status> {
    // TODO: Refactor, ideally we should create a custom CRD to track the resources we created
    // this would also simplify the "expiration" logic as the controller would just need to delete the CRD
    // and all dependent resources would be cascade deleted
    // but for right now, this will do.
    let api_sts: Api<StatefulSet> = Api::namespaced(context.kubernetes_client.clone(), "moonscale"); // TODO: Variabilize namespace
    let managed_sts = api_sts
        .list(&ListParams::default().labels("app.kubernetes.io/managed-by=Moonscale"))
        .await;
    let mut managed_dbs = Vec::<DatabaseInstanceModel>::new();

    if managed_sts.is_err() {
        return Err(Status::InternalServerError);
    }

    for sts in managed_sts.unwrap().items {
        let database_name = sts.metadata.name.clone();

        info!("Found managed database: {:?}", database_name);
        // Note: unwrapping the labels is safe here because managed_sts filtered by label.
        let db_instance_labels = sts.metadata.labels.clone().unwrap();
        let db_instance_name = db_instance_labels.get("app.kubernetes.io/instance");

        for (key, value) in db_instance_labels.iter() {
            debug!("Label: {} => {}", key, value);
        }

        if db_instance_name.is_none() {
            debug!(
                "Found managed database without instance label: {:?}",
                database_name
            );
            continue;
        }

        let db_root_password =
            get_database_password(&context.kubernetes_client, db_instance_name.unwrap()).await;

        if db_root_password.is_err() {
            debug!(
                "Failed to get database password for: {:?}",
                db_instance_name.unwrap()
            );
            continue;
        }
        managed_dbs.push(DatabaseInstanceModel {
            planetscale_api_url: format!(
                "https://{}.{}",
                db_instance_name.unwrap(),
                context.config.ingress_domain
            ),
            database_username: "root".to_owned(),
            database_password: db_root_password.unwrap(),
            database_name: db_instance_name.unwrap().to_string(),
        })
    }

    Ok(status::Custom(Status::Ok, Json(managed_dbs)))
}
