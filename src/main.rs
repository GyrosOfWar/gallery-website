use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Extension, Router,
};
use sqlx::SqlitePool;
use std::{net::SocketAddr, sync::Arc};
use tera::Tera;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

pub struct Context {
    pub tera: Tera,
    pub sql: SqlitePool,
}

impl Context {
    pub fn render_template(&self, template_name: &str) -> impl IntoResponse {
        let context = tera::Context::new();
        let output = self.tera.render(template_name, &context).unwrap();

        Html(output)
    }
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn index(Extension(context): Extension<Arc<Context>>) -> impl IntoResponse {
    context.render_template("index.html")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();

    let tera = Tera::new("templates/**/*")?;
    let sql = SqlitePool::connect("sqlite::memory").await?;
    let context = Arc::new(Context { tera, sql });
    let app = Router::new()
        .route("/", get(index))
        .fallback(get_service(ServeDir::new("public")).handle_error(handle_error))
        .layer(Extension(context))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
