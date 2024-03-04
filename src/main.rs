use rocket::form::FromForm;
use rocket::http::Status;
use rocket::{get, post, serde::json::Json};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use serde::{Deserialize, Serialize};

/// # Get if service is ready
///
/// 200 if service is ready, used for kubernetes probes
#[openapi(tag = "Kubernetes Probes")]
#[get("/readyz")]
fn readyz_route() -> Status {
    Status::Ok
}

//fn multidoc_deserialize(data: &str) -> Result<Vec<serde_yaml::Value>> {
//    let mut docs = vec![];
//
//    for de in serde_yaml::Deserializer::from_str(data) {
//        docs.push(serde_yaml::Value::deserialize(de)?);
//    }
//    Ok(docs)
//}

//fn load_database_template_old(path: &std::path::Path) -> Result<(), ()> {
//    let yaml = std::fs::read_to_string(path)
//        .with_context(|| format!("Failed to read {}", path.display()))?;
//
//    for doc in multidoc_deserialize(&yaml)? {
//        let obj: DynamicObject = serde_yaml::from_value(doc)?;
//        let namespace = obj.metadata.namespace.as_deref().or(Some("default"));
//        let gvk = if let Some(tm) = &obj.types {
//            GroupVersionKind::try_from(tm)
//        } else {
//            panic!("No type metadata in {:?}", obj);
//        };
//        let name = obj.name_any();
//
//        if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
//            let api = dynamic_api(ar, caps, client.clone(), namespace, false);
//            // trace!("Applying {}: \n{}", gvk.kind, serde_yaml::to_string(&obj)?);
//            let data: serde_json::Value = serde_json::to_value(&obj)?;
//            let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await?;
//            // info!("applied {} {}", gvk.kind, name);
//        } else {
//            // warn!("Cannot apply document for unknown {:?}", gvk);
//        }
//    }
//    Ok(())
//}

fn setup_logger() -> Result<(), log::SetLoggerError> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
}
#[rocket::main]
async fn main() {
    let launch_result = rocket::build()
        .mount("/api", openapi_get_routes![readyz_route])
        .mount(
            "/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/", openapi_get_routes![readyz_route])
        .launch()
        .await;
    match launch_result {
        Ok(_) => println!("Rocket shut down gracefully."),
        Err(err) => println!("Rocket had an error: {}", err),
    };
}
