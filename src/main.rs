use api::user_posts_api::{build_oauth_client, PostsApi};
use poem::{listener::TcpListener, middleware::AddData, session::{CookieConfig, CookieSession}, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use sqlx::{postgres::PgPool, Pool, Postgres};
use ui::UiApi;
use std::env;
mod api;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = match connect_postgresql_server().await {
        Ok(pool) => pool,
        Err(err) => {
            println!("an error occurred when connecting to database");
            return Err(err);
        }
    };
    
    let port = match env::var("PORT") {
        Ok(port) => port,
        Err(err) => {
            println!("could not find $PORT defaulting to 3000: {}", err);
            "3000".to_string()
        }
    };
    start_server(port, pool).await?;
    Ok(())
}

async fn start_server(port: String, pool: Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let client_id = env::var("CLIENT_ID").unwrap();
    let client_secret = env::var("CLIENT_SECRET").unwrap();
    println!("client_id: {}, client_secret: {}", client_id, client_secret);
    let api_service = OpenApiService::new((PostsApi, UiApi), "Hello World", "1.0")
        .server(format!("http://3.129.66.167/"));
    let api_service_docs = api_service.swagger_ui();
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", api_service_docs)
        // .nest("/", ui_service)
        .nest(
            "/static",
            poem::endpoint::StaticFilesEndpoint::new("./css").show_files_listing(),
        )
        .data(pool)
        // .with(poem::middleware::Cors::new().allow_method(poem::http::Method::GET).allow_method(poem::http::Method::POST).allow_credentials(true))
        .with(AddData::new(build_oauth_client(client_id, client_secret)))
        // .with(CookieJarManager::new());
        .with(CookieSession::new(CookieConfig::default().secure(false)));

    println!("server started!");
    Server::new(TcpListener::bind(format!("127.0.0.1:{port}")))
        .run(app)
        .await?;
    

    Ok(())
}

async fn connect_postgresql_server() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    let url = env::var("DATABASE_URL").unwrap();
    println!("database_url: {url}");
    // let url = "postgres://postgres:password@localhost:5432/new_database";
    let pool: Pool<Postgres> = tokio::time::timeout(tokio::time::Duration::from_secs(2), PgPool::connect(&url)).await??;
    sqlx::migrate!("./migrations")
        .run(&pool).await?;
    Ok(pool)
}
