use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Extension, Router,
};
use color_eyre::Report;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
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

fn get_exif_info() -> Result<Vec<HashMap<String, String>>, Report> {
    use std::{fs, io};

    let mut data = vec![];
    let files = fs::read_dir("./public/images")?;
    for entry in files {
        let entry = entry?;
        let file = fs::File::open(entry.path())?;
        let mut file = io::BufReader::new(file);
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut file)?;
        let mut kv = HashMap::new();
        for f in exif.fields() {
            kv.insert(
                f.tag.to_string(),
                f.display_value().with_unit(&exif).to_string(),
            );
        }
        data.push(kv);
    }

    Ok(data)
}

async fn index(Extension(context): Extension<Arc<Context>>) -> impl IntoResponse {
    let info = get_exif_info().unwrap();
    println!("{:#?}", info);

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
    let sql =
        SqlitePool::connect_with(SqliteConnectOptions::new().filename("gallery.sqlite3")).await?;
    sqlx::migrate!("./migrations").run(&sql).await?;

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
