use api::user_posts_api::PostsApi;
use poem::{listener::TcpListener, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use sqlx::{postgres::PgPool, Pool, Postgres};
mod api;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = sql_postgres().await?;    
    server(pool).await?;
    Ok(())
}


 
async fn server(pool: Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let api_service =
        OpenApiService::new(PostsApi, "Hello World", "1.0")
        .server("http://localhost:3000/api");

    let ui_service = ui::get_service().server("http://localhost:3000/");
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

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?;
    Ok(())
}

async fn sql_postgres() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    let url: &str = "postgres://postgres:password@localhost:5432/new_database";
    let pool: Pool<Postgres> = PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}