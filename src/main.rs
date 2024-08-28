use api::user_posts_api::{self, PostsApi};
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use sqlx::{postgres::PgPool, Pool, Postgres};
use std::env;
mod api;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = sql_postgres().await?;    
    server(pool).await?;
    Ok(())
}

async fn server(pool: Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    
    let port = match env::var("PORT") {
        Ok(port) => port,
        Err(err) => {
            println!("could not find $PORT defaulting to 3000: {}", err);
            "3000".to_string()
        }
    };

    let api_service: OpenApiService<PostsApi, ()> = user_posts_api::get_service()
        .server(format!("https://localhost:{port}/api"));
    let ui_service: OpenApiService<ui::UiApi, ()> = ui::get_service()
        .server(format!("https://localhost:{port}/"));
    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest("/api", api_service)
        .nest("api/docs", ui)
        .nest("/", ui_service)
        .nest(
            "/static",
            poem::endpoint::StaticFilesEndpoint::new("./css/").show_files_listing(),
        )
        .data(pool);

    Server::new(TcpListener::bind(format!("127.0.0.1:{port}")))
        .run(app)
        .await?;
    Ok(())
}

async fn sql_postgres() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    // let url = env::var("DATABASE_URL").unwrap();
    
    let url = "postgres://postgres:password@localhost:5432/new_database";
    let pool: Pool<Postgres> = PgPool::connect(url).await?;
    sqlx::migrate!("./migrations")
        .run(&pool).await?;
    Ok(pool)
}