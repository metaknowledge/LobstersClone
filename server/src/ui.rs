use askama::Template;
use poem::web::Data;
use poem::{IntoResponse, Response};
use poem_openapi::param::{Path, Query};
use poem_openapi::{ApiResponse, Object, OpenApiService, ResponseContent};
use poem_openapi::{payload::PlainText, OpenApi};
use sqlx::{Pool, Postgres};
use poem_openapi::payload::{Html, Json};


#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    pub postid: i64
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTemplate;

#[derive(Template)]
#[template(path = "user.html")]
struct UserTempate {
    pub username: String,
}

#[derive(ApiResponse)]
pub enum UiReponse {
    #[oai(status = 200)]
    Ok(Html<String>),

    #[oai(status = 400)]
    InvalidRequest,

    #[oai(status = 404)]
    NotFound,
}

pub struct UiApi;

#[OpenApi]
impl UiApi {
    #[oai(path="/post/:id", method="get")]
    async fn post(
        &self,
        Path(id): Path<Option<i64>>,
    ) -> Html<String> {       
        let post = PostTemplate{postid: id.unwrap()}.render().map_err(poem::error::InternalServerError).unwrap();
        Html(post)
    }

    #[oai(path="/", method="get")]
    async fn home(
        &self,
    ) -> Html<String> {
        let home = HomeTemplate.render().map_err(poem::error::InternalServerError).unwrap();
        Html(home)
    }

    #[oai(path="/signup/", method="get")]
    async fn signup(
        &self,
    ) -> Html<String> {
        let home = SignupTemplate.render().map_err(poem::error::InternalServerError).unwrap();
        Html(home)
    }

    #[oai(path="/user/:user", method="get")]
    async fn user(
        &self,
        Path(user): Path<String>,
    ) -> Html<String> {
        let user = UserTempate{username: user}.render().map_err(poem::error::InternalServerError).unwrap();
        Html(user)
    }
}

pub fn get_service() -> OpenApiService<UiApi, ()> {
    let api_service: OpenApiService<UiApi, ()> =
        OpenApiService::new(UiApi, "Hello World", "1.0");
    api_service
    
}

// cite: https://github.com/nicolasauler/anodized-poem/blob/main/src/main.rs
struct HtmlTemplate<T>(T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template + Send + Sync + 'static,
{
    fn into_response(self) -> poem::Response {
        let body = self.0.render().unwrap();
        poem::Response::builder()
            .content_type("text/html; charset=utf-8")
            .body(body)
    }
}