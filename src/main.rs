use axum::{
    body,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use color_eyre::Report;
use std::{fmt, net::SocketAddr, sync::Arc};
use tera::Tera;
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct Context {
    pub tera: Tera,
}

impl Context {
    pub fn render_template(&self, template_name: &str) -> impl IntoResponse {
        let context = tera::Context::new();
        let output = self.tera.render(template_name, &context).unwrap();

        Html(output)
    }
}

async fn index(Extension(context): Extension<Arc<Context>>) -> impl IntoResponse {
    context.render_template("index.html")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let tera = Tera::new("templates/**/*")?;
    let context = Arc::new(Context { tera });
    let app = Router::new()
        .route("/", get(index))
        .layer(Extension(context));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
