use askama::Template;
use chrono::Local;
use poem::session::{self, Session};
use poem::web::Data;
use poem_openapi::param::{Path, Query};
use poem_openapi::ApiResponse;
use poem_openapi::Object;
use poem_openapi::{payload::PlainText, OpenApi};
use reqwest::Response;
use sqlx::{Pool, Postgres};
use poem_openapi::payload::{Html, Json};
use crate::api::users;
use crate::api::sessions;
use crate::api::posts::{self, Post};
use crate::api::routes::{UserDeleteResponse, Info};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, Scope, TokenResponse, TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2;
use super::routes::{CreatePostReponse, UserApiResponse};
pub struct PostsApi;

use poem::session::CookieConfig;


#[derive(Object, Clone)]
pub struct CreatePost {
    pub username: String,
    pub title: String,
    pub content: String,
}

#[derive(Object, Clone, Default)]
pub struct UpdatePost {
    pub title: String,
    pub content: String,
}

#[derive(Object, Clone)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "posts.html")]
struct PostsTemplate {
    pub posts: Vec<Post>,
    pub page: i64,
    pub editable: bool,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    pub post: Post,
    pub editable: bool,
    pub i: i32
}

#[derive(Template)]
#[template(path = "editpost.html")]
struct EditPostTemplate {
    pub postid: i64,
    pub title: String,
    pub content: String,
}

pub fn build_oauth_client(client_id: String, client_secret: String) -> BasicClient {
    let redirect_url = "http://localhost:3000/api/auth/discord/redirect".to_string();
    
    let auth_url = AuthUrl::new("https://discord.com/oauth2/authorize".to_string())
        .expect("Wrong auth endpoint");
    let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
        .expect("Wrong token url");
    
    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

pub async fn check_user_creds(session: &Session, pool: &Pool<Postgres>) -> Result<sessions::UserProfile, ApiAuthResponse> {
    let session_id = match session.get::<String>(SID) {
        Some(cookie) => cookie,
        // None => return Err(ApiAuthResponse::Redirect("/signup".to_string())),
        None => return Err(ApiAuthResponse::Redirect("/signup".to_string()))
    };
    let res = match sessions::get(session_id, pool).await {
        Ok(result) => result,
        Err(err) => return Err(ApiAuthResponse::NotAuthorized)
    };
    Ok(res)
}

pub const SID: &str = "sid";


#[derive(ApiResponse)]
enum RedirectResponse {
    #[oai(status = "307")]
    Redirect(#[oai(header = "Location")] String),
    #[oai(status = 400)]
    InvalidRequest(PlainText<String>),
}

#[derive(ApiResponse)]
pub enum ApiAuthResponse {
    #[oai(status = 200)]
    Ok(Html<String>),
    #[oai(status = 302)]
    Redirect(#[oai(header = "Location")] String),
    #[oai(status = 400)]
    InvalidRequest(Html<String>),
    #[oai(status = 404)]
    NotFound,
    #[oai(status = 401)]
    NotAuthorized,
}


// #[OpenApi]
#[OpenApi(prefix_path = "/api")]
impl PostsApi {
    #[oai(path="/auth/discord/redirect", method="get")]
    async fn discord_auth(
        &self,
        Query(code): Query<String>,
        Data(pool): Data<&Pool<Postgres>>,
        Data(middle): Data<&BasicClient>,
        // cookie_jar: &CookieJar
        session: &Session,
    ) -> ApiAuthResponse {
        
        // let client: BasicClient = build_oauth_client(client_id, client_secret);
        let client: BasicClient = middle.clone();
        let token = client.exchange_code(AuthorizationCode::new(code.clone()))
            .request_async(oauth2::reqwest::async_http_client)
            .await.expect("should have got code");
        // println!("done".to_string());
        
        
        let ctx = reqwest::Client::new();
        let response: Response = ctx.get("https://discord.com/api/v10/users/@me")
            .bearer_auth(token.access_token().secret().to_owned())
            .send().await.unwrap();
        let profile = response.json::<sessions::UserProfile>().await.unwrap();
        let Some(secs) = token.expires_in() else {

            return ApiAuthResponse::InvalidRequest(Html("<p>could not find token expiration<p>".to_string()))
        };

        let secs = secs.as_secs();

        let max_age = Local::now() + chrono::Duration::try_seconds(secs.try_into().unwrap()).unwrap();
        
        // creates cookie
        let cookie = CookieConfig::default()
            .name(SID)
            .domain("localhost")
            .max_age(core::time::Duration::from_secs(secs));
        
        session.set(SID, token.access_token().secret());
        
        // cookie.set_cookie_value(cookie_jar, token.access_token().secret()); // adds it to the cookie jar
        // println!("{}", cookie_jar.get(SID).unwrap().to_string());

        // println!("{}", cookie.get_cookie_value(cookie_jar).unwrap());

        let _userid = sqlx::query!("insert into users (email, username) values ($1, $2) on conflict (username) do nothing;",
            profile.email, profile.username).execute(pool).await.unwrap();


        let _test = sqlx::query!(
                "INSERT INTO sessions (user_id, session_id, expires_at) VALUES (
                (SELECT ID FROM USERS WHERE email = $1 LIMIT 1),
                 $2, $3)
                ON CONFLICT (user_id) DO UPDATE SET
                session_id = excluded.session_id,
                expires_at = excluded.expires_at;",
                profile.email,
                token.access_token().secret().to_owned(),
                max_age
            )
            .execute(pool)
            .await.unwrap();

        ApiAuthResponse::Redirect("/api/protected".to_string())
    }

    #[oai(path="/protected", method="get")]
    async fn protected(
        &self,
        session: &Session,
        Data(pool): Data<&Pool<Postgres>>,
    ) -> ApiAuthResponse {
        let res = match check_user_creds(session, pool).await {
            Ok(res) => res,
            Err(err) => return err,
        };
        ApiAuthResponse::Ok(
            Html(format!("<p>email:{}, username:{}<p>", res.email, res.username))
        )
    }

    #[oai(path="/html/posts", method="get")]
    async fn get_paged_html(
        &self,
        Query(page): Query<Option<i64>>,
        pool: Data<&Pool<Postgres>>
    ) -> Html<String> {
        let posts: Vec<Post> = posts::read_page_number(page.unwrap(), &pool).await.unwrap();  
        if posts.len() == 0 {
            return Html("You made it to the bottom".to_string())
        }
        let html = PostsTemplate {posts: posts, page: page.unwrap() + 1, editable: false}
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }


    // get one post
    #[oai(path="/post/:post_id", method="get")]
    async fn get_post_from_id(
        &self,
        Path(post_id): Path<String>,
        pool: Data<&Pool<Postgres>>,
    ) -> PlainText<String> {
        let post = posts::read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        PlainText(post.to_string())
    }

    // create post
    #[oai(path="/post", method="post")]
    async fn post_post(
        &self,
        pool: Data<&Pool<Postgres>>,
        create_post: Json<CreatePost>
    ) -> CreatePostReponse {
        let title = create_post.title.clone();
        let content = create_post.content.clone();
        let username = create_post.username.clone();
        println!("{}{}{}", title, content, username);
        let result = posts::create(title, content, username, &pool).await;
        match result {
            Ok(postid) => CreatePostReponse::Ok(Info::Info(PlainText(postid.to_string()))),
            Err(err) => CreatePostReponse::InvalidRequest(Info::Info(PlainText(err.to_string() + ": An error has occured")))
        }
    }

    // delete post
    #[oai(path="/post/:post_id", method="delete")]
    async fn delete_post(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(post_id): Path<i32>,
    ) -> PlainText<String> {
        let _ = posts::delete(post_id, &pool).await;        
        PlainText("deleted".to_string())  
    }

    // update post
    #[oai(path="/post/:post_id", method="put")]
    async fn update_post(
        &self,
        Path(post_id): Path<i32>,
        update_post: Json<UpdatePost>,
        session: &Session,
        Data(pool): Data<&Pool<Postgres>>,
    ) -> ApiAuthResponse {
        let user: sessions::UserProfile = match check_user_creds(session, pool).await {
            Ok(res) => res,
            Err(err) => return err,
        };
        if !posts::check_if_user_can_edit_post(user.username, post_id, pool).await.unwrap() {
            return ApiAuthResponse::NotAuthorized;
        }
        let title = update_post.title.clone();
        let content = update_post.content.clone();
        let result = posts::update(title.clone(), content.clone(), post_id, &pool).await.unwrap();        
        match result.rows_affected() {
            0 => ApiAuthResponse::InvalidRequest(Html("could not update row".to_string())),
            _ => {
                let post = Post {
                    title: title,
                    id: post_id,
                    username: String::new(),
                    content: content
                };
                let html = PostTemplate {post: post, editable: true, i: 0}
                    .render()
                    .map_err(poem::error::InternalServerError)
                    .unwrap();
                ApiAuthResponse::Ok(Html(html))
            }
        }
    }

    #[oai(path="/post/:post_id/edit", method="get")]
    async fn edit_post_html(
        &self,
        Path(post_id): Path<String>,
        pool: Data<&Pool<Postgres>>,
    ) -> Html<String> {
        let post = posts::read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        let html = EditPostTemplate {postid: post_id.parse::<i64>().unwrap(), title: post.title, content: post.content }
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }

    // get one user
    #[oai(path="/user/:maybe_username", method="get")]
    async fn get_user_from_id(
        &self,
        Path(username): Path<String>,
        Data(pool): Data<&Pool<Postgres>>,
    ) -> PlainText<String> {
        
        let user = users::read_username(username, &pool).await; 
        match user {
            Ok(user) => PlainText(user.to_string()),
            Err(error) => PlainText(error.to_string() + ": Error please fix")
        }
    }

    // get posts associated with user: username
    #[oai(path="/user/posts/:username", method="get")]
    async fn get_user_posts(
        &self,
        Path(username): Path<String>,
        Query(page_number): Query<Option<i64>>,
        session: &Session,
        Data(pool): Data<&Pool<Postgres>>,
    ) -> Html<String> {
        let page = match page_number {
            Some(page) => page,
            None => 0
        };
        let editable = match check_user_creds(session, pool).await {
            Ok(res) => res.username == username,
            Err(_) => false
        };
        let posts: Vec<Post> = posts::get_posts_from_user(username.clone(), page, pool).await.unwrap();        
        if posts.len() == 0 {
            return Html("You made it to the bottom".to_string())
        }
        let html = PostsTemplate {posts: posts, page: page + 1, editable: editable}
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }

    // create user
    #[oai(path="/user", method="post")]
    async fn create_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        req: Json<CreateUser>
    ) -> PlainText<String> {
        let user_id = users::create(req.username.clone(), req.email.clone(), &pool).await;
        PlainText(user_id.unwrap().to_string())
    }

    // update user
    #[oai(path="/user/:user_id", method="put")]
    async fn update_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(user_id): Path<i32>,
        Query(username): Query<Option<String>>,
        Query(email): Query<Option<String>>,
    ) -> PlainText<String> {
        let result = users::update(username.unwrap(), email.unwrap(), user_id, &pool).await;        
        match result {
            Ok(_) => PlainText("updated".to_string()),  
            Err(_) => PlainText("could not update".to_string()),
        }
    }

    // delete user
    #[oai(path="/user/:user_id", method="delete")]
    async fn delete_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(user_id): Path<i32>,
    ) -> UserDeleteResponse {
        let result = users::delete(user_id, &pool).await;
        match result {
            Ok(_) => UserDeleteResponse::Ok(Info::Info(PlainText(result.unwrap().username))),
            Err(_e) => UserDeleteResponse::NotFound
        }
    }


}