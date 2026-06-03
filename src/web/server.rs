use axum::{response::{Html, Response}, routing::{get, post}, Router, body::Body};
use std::net::SocketAddr;

use super::routes::{dependencies, open, projects, stats};

const DASHBOARD_HTML: &str = include_str!("dashboard.html");
const PROJECTS_HTML: &str = include_str!("projects.html");
const PROJECT_DETAIL_HTML: &str = include_str!("project_detail.html");
const DEPENDENCIES_HTML: &str = include_str!("dependencies.html");
const I18N_JS: &str = include_str!("i18n.js");

pub async fn run(port: u16, open_browser: bool) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    assert!(addr.ip().is_loopback(), "must bind to loopback");

    let app = Router::new()
        .route("/", get(|| async { Html(DASHBOARD_HTML) }))
        .route("/projects", get(|| async { Html(PROJECTS_HTML) }))
        .route("/project/:name", get(|| async { Html(PROJECT_DETAIL_HTML) }))
        .route("/dependencies", get(|| async { Html(DEPENDENCIES_HTML) }))
        .route("/i18n.js", get(|| async {
            Response::builder()
                .header("Content-Type", "application/javascript; charset=utf-8")
                .body(Body::from(I18N_JS))
                .unwrap()
        }))
        .route("/api/stats", get(stats::get_stats))
        .route("/api/projects", get(projects::get_projects))
        .route("/api/projects/:name", get(projects::get_project_by_name))
        .route("/api/dependencies", get(dependencies::get_dependencies))
        .route("/api/open", post(open::post_open));

    if open_browser {
        let url = format!("http://{}", addr);
        let _ = webbrowser::open(&url);
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("pyforge web listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("shutting down...");
}
