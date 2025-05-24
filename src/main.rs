use std::{env, path::PathBuf};

use axum::{extract::State, http::StatusCode, response::Html, routing::get, Router};
use reqwest::header;
use serde::{Deserialize, Serialize};
use tower_http::{services::ServeFile, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const BULB_ON_DIV: &str = "<div class='container' style='background-color:#F5DEB3;'><img alt='Lightbulb on' src='/bulbon'/></div>";
const BULB_OFF_DIV: &str = "<div class='container' style='background-color:black;'><img alt='Lightbulb off' src='/bulboff'/></div>";

#[derive(Deserialize, Debug, Serialize)]
struct LightState {
    state: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct Entity {
    entity_id: String,
}

fn create_reqwest_client() -> reqwest::Client {
    let yagami_token = env::var("YAGAMI_TOKEN").expect("YAGAMI_TOKEN should be set");

    let mut headers = header::HeaderMap::new();

    let mut token = header::HeaderValue::from_str(&format!("Bearer {yagami_token}"))
        .expect("This should be a valid string");
    token.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, token);
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("client should be buildable")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "yagami=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client = create_reqwest_client();
    let public_path = PathBuf::from(
        env::var("YAGAMI_PUBLIC_PATH").unwrap_or(String::from("/var/lib/yagami/public")),
    );

    // Read LIGHT_ID once at startup
    let light_id = env::var("LIGHT_ID").expect("LIGHT_ID should be set");

    let app = Router::new()
        .route("/", get(index))
        .route("/bulb", get(get_state).post(swap_state))
        .route("/alive", get(alive))
        .route_service(
            "/bulbon",
            ServeFile::new(format!("{}/on.png", public_path.to_string_lossy())),
        )
        .route_service(
            "/bulboff",
            ServeFile::new(format!("{}/off.png", public_path.to_string_lossy())),
        )
        .layer(TraceLayer::new_for_http())
        .with_state((client, light_id));

    let port = env::var("YAGAMI_PORT").unwrap_or(String::from("2802"));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap_or_else(|_| panic!("Could not listen on port {port}"));
    tracing::info!(
        "Listening on {}",
        listener
            .local_addr()
            .expect("We should be able to get the URL we are listening on")
    );

    // Notify systemd that the service is ready if running under systemd
    let is_systemd = env::var("NOTIFY_SOCKET").is_ok();
    if is_systemd {
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("This should run until the end of the program");
        });

        // Send notification to systemd
        let _ = tokio::process::Command::new("systemctl")
            .arg("--no-pager")
            .arg("--no-ask-password")
            .arg("notify")
            .arg("READY=1")
            .status()
            .await;
    } else {
        axum::serve(listener, app)
            .await
            .expect("This should run until the end of the program");
    }
}

async fn index() -> Html<&'static str> {
    tracing::info!("Request on index");
    Html(std::include_str!("../public/index.html"))
}

async fn alive() -> (StatusCode, &'static str) {
    (StatusCode::OK, "yagami is alive!")
}

async fn get_state(
    State((client, light_id)): State<(reqwest::Client, String)>,
) -> (StatusCode, &'static str) {
    tracing::info!("get state");
    let current_state: LightState = client
        .get(format!("http://192.168.1.108:8123/api/states/{}", light_id))
        .send()
        .await
        .expect("We should have a response")
        .json::<LightState>()
        .await
        .expect("Should contain something");

    if current_state.state == *"on" {
        (StatusCode::OK, BULB_ON_DIV)
    } else {
        (StatusCode::OK, BULB_OFF_DIV)
    }
}

async fn swap_state(
    State((client, light_id)): State<(reqwest::Client, String)>,
) -> (StatusCode, &'static str) {
    tracing::info!("set state");

    let current_state: LightState = client
        .get(format!("http://192.168.1.108:8123/api/states/{}", light_id))
        .send()
        .await
        .expect("We should have a response")
        .json::<LightState>()
        .await
        .expect("Should contain something");

    let light = Entity {
        entity_id: light_id,
    };

    if current_state.state == *"on" {
        let response = client
            .post("http://192.168.1.108:8123/api/services/light/turn_off")
            .json(&light)
            .send()
            .await
            .expect("We should be able to post the new state");
        (response.status(), BULB_OFF_DIV)
    } else {
        let response = client
            .post("http://192.168.1.108:8123/api/services/light/turn_on")
            .json(&light)
            .send()
            .await
            .expect("We should be able to post the new state");
        (response.status(), BULB_ON_DIV)
    }
}
