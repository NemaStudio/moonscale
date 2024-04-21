use crate::middlewares::authentication::ApiKey;
use kube::{
    api::{DeleteParams, DynamicObject, ListParams},
    discovery::{verbs, Scope},
    Api, Discovery, ResourceExt,
};
use log::{info, warn};
use rocket::{delete, http::Status, State};
use rocket_okapi::openapi;

async fn delete_database(
    instance: &str,
    context: &crate::context::Context,
) -> Result<(), anyhow::Error> {
    let discovery = Discovery::new(context.kubernetes_client.clone())
        .run()
        .await?;
    let mut deleted: bool = false;

    for group in discovery.groups() {
        for (ar, caps) in group.recommended_resources() {
            if !caps.supports_operation(verbs::DELETE) || !caps.supports_operation(verbs::LIST) {
                continue;
            }
            let api: Api<DynamicObject> = if caps.scope == Scope::Cluster {
                Api::all_with(context.kubernetes_client.clone(), &ar)
            } else {
                Api::namespaced_with(context.kubernetes_client.clone(), "moonscale", &ar)
            };

            let list = api
                .list(
                    &ListParams::default().labels(
                        format!(
                            "app.kubernetes.io/managed-by=Moonscale,app.kubernetes.io/instance={}",
                            instance
                        )
                        .as_str(),
                    ), //.labels(format!("app.kubernetes.io/managed-by=Moonscale").as_str()
                )
                .await;

            if list.is_err() {
                warn!(
                    "Failed to list resources while deleting instance {} ({}/{})",
                    instance,
                    ar.kind,
                    list.err().unwrap()
                );
                continue;
            }

            for item in list.unwrap().items {
                let name = item.name_any();
                let ns = item.metadata.namespace.map(|s| s + "/").unwrap_or_default();

                info!(
                    "Deleting resource: namespace: {}, resource: {}, name: {}",
                    ns, ar.kind, name
                );
                let delete_result = api.delete(name.as_str(), &DeleteParams::foreground()).await;

                if delete_result.is_err() {
                    warn!(
                        "Failed to cleanup resource while deleting instance {} ({}/{})",
                        instance, ar.kind, name
                    );
                } else {
                    deleted = true;
                }
            }
        }
    }

    match deleted {
        true => Ok(()),
        false => {
            warn!(
                "No resources found for instance {}, or couldn't delete them skipping delete order.",
                instance
            );
            Err(anyhow::anyhow!(
                "No resources found for instance {}",
                instance
            ))
        }
    }
}

/// # Delete a managed database
///
/// This route is used to delete a deployed moonscale database.
#[openapi(tag = "Database")]
#[delete("/database/<instance>")]
pub async fn route_delete_database(
    instance: &str,
    context: &State<crate::context::Context>,
    _key: ApiKey,
) -> Status {
    info!("Deleting moonscale instance {}", instance);
    let delete_result = delete_database(instance, context).await;

    if delete_result.is_err() {
        // TODO: Refactor
        if delete_result
            .err()
            .unwrap()
            .to_string()
            .contains("No resources found")
        {
            return Status::NotFound;
        }
        return Status::InternalServerError;
    }

    Status::Ok
}
