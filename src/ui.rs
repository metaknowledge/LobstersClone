use askama::Template;
use poem::session::Session;
use poem_openapi::param::Path;
use poem_openapi::ApiResponse;
use poem_openapi::OpenApi;
use poem_openapi::payload::Html;
use sqlx::{Pool, Postgres};
use poem::web::Data;
use crate::api::posts::{self, Post};
use crate::api::user_posts_api::ApiAuthResponse;
use crate::api::users::{self, User};


#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[derive(Template)]
#[template(path = "focus_post.html")]
struct PostTemplate {
    pub post: Post
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTemplate;

#[derive(Template)]
#[template(path = "user.html")]
struct UserTempate {
    pub username: String,
}

#[derive(Template)]
#[template(path = "allusers.html")]
struct AllUsersTempate {
    pub users: Vec<User>,
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
        Path(id): Path<i32>,
        pool: Data<&Pool<Postgres>>,
    ) -> Html<String> { 
        let post = posts::read_from_id(id, &pool).await.unwrap();        
        let html: String = PostTemplate{post: post}.render().map_err(poem::error::InternalServerError).unwrap();
        Html(html)
    }

    // async fn get_post_from_id(
    //     &self,
    //     Path(post_id): Path<String>,
    //     pool: Data<&Pool<Postgres>>,
    // ) -> PlainText<String> {
    //     let post = posts::read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
    //     PlainText(post.to_string())
    // }


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
        session: &Session,
        Data(pool): Data<&Pool<Postgres>>,
    ) -> ApiAuthResponse {
        let usertemp = UserTempate{username: user}.render().map_err(poem::error::InternalServerError).unwrap();
        ApiAuthResponse::Ok(Html(usertemp))
    }

    #[oai(path="/users", method="get")]
    async fn users(
        &self,
        pool: Data<&Pool<Postgres>>

    ) -> Html<String> {
        let users: Vec<User> = users::read_all(&pool).await.unwrap();     
        let userstemp = AllUsersTempate {users: users}
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(userstemp)
    }
}