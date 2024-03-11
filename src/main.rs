use std::env;

use crate::routes::{create_database::*, delete_database::*, list_database::*};
use anyhow::Result;
use context::Config;
use kube::error;
use log::{error, info};
use rocket::get;
use rocket::http::Status;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};

mod context;
mod kubernetes;
mod middlewares;
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
        .level(if cfg!(debug_assertions) {
            // TODO: Add a flag to enable debug logging
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Debug
        })
        .chain(std::io::stdout())
        .apply()
}

fn build_config() -> Result<Config, ()> {
    let env_api_key = env::var("MOONSCALE_API_KEY");

    if env_api_key.is_err() || env_api_key.clone().unwrap() == "" {
        error!("Invalid API key, did you set the MOONSCALE_API_KEY environment variable to a non-empty string ?");
        return Err(());
    }

    Ok(Config {
        api_key: env_api_key.unwrap(),
        ingress_domain: env::var("MOONSCALE_INGRESS_DOMAIN").unwrap_or("example.com".to_owned()),
        resource_ttl: env::var("MOONSCALE_RESOURCE_TTL")
            .unwrap_or("3600".to_owned())
            .parse()
            .unwrap_or_else(|err| {
                error!("Failed to parse MOONSCALE_RESOURCE_TTL: {}", err);
                std::process::exit(1);
            }),
    })
}

#[rocket::main]
async fn main() -> Result<(), ()> {
    if setup_logger().is_err() {
        eprintln!("Failed to setup logger, exiting.");
    }
    let config = build_config();

    if config.is_err() {
        error!("Couldn't build configuration, check logs for error.");
        return Err(());
    }
    let context = context::Context {
        database_template_yaml_raw: include_str!("../resources/template.yml").to_owned(),
        kubernetes_client: kube::Client::try_default().await.unwrap_or_else(|err| {
            error!("Failed to create kubernetes client: {}", err);
            std::process::exit(1);
        }),
        config: config.unwrap(),
    };

    info!("Starting moonscale server with context:");
    info!("\tIngress domain: {}", context.config.ingress_domain);

    let launch_result = rocket::build()
        .mount(
            "/api",
            openapi_get_routes![
                route_create_database,
                route_list_database,
                route_delete_database
            ],
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
