use k8s_openapi::api::apps::v1::StatefulSet;
use kube::{api::ListParams, Api};
use log::debug;
use rocket::{get, http::Status, State};
use rocket_okapi::openapi;

/// # List all managed databases
///
/// This route is used to list deployed moonscale databases.
#[openapi(tag = "Database")]
#[get("/database")]
pub async fn route_list_database(context: &State<crate::context::Context>) -> Status {
    // TODO: Refactor, ideally we should create a custom CRD to track the resources we created
    // this would also simplify the "expiration" logic as the controller would just need to delete the CRD
    // and all dependent resources would be cascade deleted
    // but for right now, this will do.
    let api: Api<StatefulSet> = Api::all(context.kubernetes_client.clone());
    let managed_sts = api
        .list(&ListParams::default().labels("app.kubernetes.io/managed-by=Moonscale"))
        .await;

    if managed_sts.is_err() {
        return Status::InternalServerError;
    }
    for sts in managed_sts.unwrap().items {
        debug!("{:?}", sts.metadata.name.unwrap());
    }
    Status::Ok
}
