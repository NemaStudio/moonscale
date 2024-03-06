use crate::routes::{create_database::*, list_database::*};
use anyhow::Result;
use log::error;
use rocket::get;
use rocket::http::Status;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

mod context;
mod kubernetes;
mod models;
mod routes;

/// # Get if service is ready
///
/// 200 if service is ready, used for kubernetes probes
#[openapi(tag = "Kubernetes Probes")]
#[get("/readyz")]
fn readyz_route() -> Status {
    Status::Ok
}

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
async fn main() -> Result<(), ()> {
    if setup_logger().is_err() {
        eprintln!("Failed to setup logger, exiting.");
    }

    let context = context::Context {
        database_template_yaml_raw: include_str!("../resources/template.yml").to_owned(),
        kubernetes_client: kube::Client::try_default().await.unwrap_or_else(|err| {
            error!("Failed to create kubernetes client: {}", err);
            std::process::exit(1);
        }),
    };

    let launch_result = rocket::build()
        .mount(
            "/api",
            openapi_get_routes![route_create_database, route_list_database],
        )
        .mount(
            "/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/", openapi_get_routes![readyz_route])
        .manage(context)
        .launch()
        .await;
    match launch_result {
        Ok(_) => println!("Rocket shut down gracefully."),
        Err(err) => println!("Rocket had an error: {}", err),
    };
    Ok(())
}
