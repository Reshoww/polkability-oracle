use actix_web::{web, App, HttpServer};
use serde::Deserialize;
use std::time::Duration;
use std::boxed::Box;
use chrono::{Local, SecondsFormat};
use reqwest;
use tokio;

// Contract to notify.
static CONTRACT_NAME: &str = "polkability";

// JSON data structure that RPC received as event payload.
#[derive(Deserialize)]
struct ProbabilityEventJSONPayload {
    prediction_topic: String,
    resource_urls: Vec<String>
}

// JSON data structure that OnChain contract handing as payload.
#[derive(Deserialize)]
struct ChangeEventPayload {
    completion_date: String,
    topic: String
}

// Notify OnChain contract.
fn notify_runtime_contract(_args: &crate::ChangeEventPayload) {
    // todo: call a `dispatch_event` method on contract with args.
}

// Check if a global event condition is satisfied.
async fn change_condition_successfied(resource_url: &str, topic: &str) -> Result<bool, reqwest::Error> {
    let observe_current_result = reqwest::get(resource_url).await?.text().await?;

    if observe_current_result.contains(topic) {
        return Ok(true);
    }

    return Ok(false);
}

// Observe change on resources.
async fn run_change_observation(time_interval_in_mills: u64, resource_urls: Vec<String>, observation_topic: String) {
    let topic: &'static str = Box::leak(observation_topic.into_boxed_str());

    for resource_url in resource_urls {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(time_interval_in_mills));

            loop {
                interval.tick().await;

                let is_successfull = change_condition_successfied(&resource_url.clone(), topic).await.unwrap();
                let time_now: String = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    
                println!("change condition satisfied: {} at date {}", is_successfull, time_now);
    
                let change_event_args = &crate::ChangeEventPayload {
                    completion_date: time_now.to_string(),
                    topic: topic.to_string()
                };
    
                if is_successfull {
                    notify_runtime_contract(&change_event_args);
                }
            }
        });
    }
}

// Handle post OffChain event.
async fn post_observe_event(event_data: web::Json<ProbabilityEventJSONPayload>) -> std::string::String {
    run_change_observation(1000, event_data.resource_urls.clone(), event_data.prediction_topic.clone()).await;
    format!("event topic: {}", event_data.prediction_topic)
}

// Return current API version.
async fn get_api_version() -> std::string::String {
    format!("api version: {}", 1)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/event", web::post().to(post_observe_event))
            .route("/api/version", web::get().to(get_api_version))
    })
        .bind(("127.0.0.1", 3000))?.run().await
}
