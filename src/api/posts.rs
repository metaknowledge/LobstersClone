use std::fmt::Display;

use askama::Template;

use poem_openapi::OpenApi;
use poem_openapi::payload::{Html, Json, PlainText};
use poem_openapi::param::{Path, Query};
use poem::session::Session;
use poem::web::Data;

use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use sqlx::{Pool, Postgres};

use crate::api::utils::{ApiAuthResponse, check_user_creds};
use crate::api::routes;
use crate::api::sessions;

#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct Post {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub content: String,
    #[sqlx(default)]
    pub link: Option<String>
}

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

#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostsTemplate {
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

impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "USERNAME: {}, TITLE: {}, CONTENT: {}", self.username, self.title, self.content)
    }
}

pub struct PostsApi;

#[OpenApi(prefix_path = "/api")]
impl PostsApi {
    // queries the database based on the page number, pages are 10 posts long
    #[oai(path="/html/posts", method="get")]
    async fn get_paged_html(
        &self,
        Query(page): Query<Option<i64>>,
        pool: Data<&Pool<Postgres>>
    ) -> Html<String> {
        let posts: Vec<Post> = read_page_number(page.unwrap(), &pool).await.unwrap();  
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
        let post = read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        PlainText(post.to_string())
    }

    // create post
    #[oai(path="/post", method="post")]
    async fn post_post(
        &self,
        pool: Data<&Pool<Postgres>>,
        create_post: Json<CreatePost>
    ) -> routes::CreatePostReponse {
        let title = create_post.title.clone();
        let content = create_post.content.clone();
        let username = create_post.username.clone();
        println!("{}{}{}", title, content, username);
        let result = create(title, content, username, &pool).await;
        match result {
            Ok(postid) => routes::CreatePostReponse::Ok(routes::Info::Info(PlainText(postid.to_string()))),
            Err(err) => routes::CreatePostReponse::InvalidRequest(routes::Info::Info(PlainText(err.to_string() + ": An error has occured")))
        }
    }

    // delete post
    #[oai(path="/post/:post_id", method="delete")]
    async fn delete_post(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(post_id): Path<i32>,
    ) -> PlainText<String> {
        let _ = delete(post_id, &pool).await;        
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
        if !check_if_user_can_edit_post(user.username, post_id, pool).await.unwrap() {
            return ApiAuthResponse::NotAuthorized;
        }
        let title = update_post.title.clone();
        let content = update_post.content.clone();
        let result = update(title.clone(), content.clone(), post_id, &pool).await.unwrap();        
        match result.rows_affected() {
            0 => ApiAuthResponse::InvalidRequest(Html("could not update row".to_string())),
            _ => {
                let post = Post {
                    title: title,
                    id: post_id,
                    username: String::new(),
                    content: content,
                    ..Default::default()
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
        let post = read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        let html = EditPostTemplate {postid: post_id.parse::<i64>().unwrap(), title: post.title, content: post.content }
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }
}

pub async fn create(title: String, content: String, username: String, pool: &sqlx::PgPool) -> Result<i32, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!(
                "INSERT INTO posts (title, content, user_id)
                VALUES ($1, $2, (SELECT u.id FROM users u WHERE username=$3))
                RETURNING id;",
                title,
                content,
                username
            )
            .fetch_one(pool)
            .await?;
    Ok(result.id)
}

pub async fn update(title: String, content: String, post_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query_as!(Post, "UPDATE posts set title=$1, content=$2 WHERE id=$3;", title, content, post_id)
            .execute(pool)
            .await?;
    Ok(result)
}

pub async fn delete(post_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!("DELETE FROM posts WHERE id=$1;", post_id)
            .execute(pool)
            .await?;
    Ok(result)
}

pub async fn read_page_number(page: i64, pool: &sqlx::PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Post,
            "SELECT p.title, p.content, p.id, u.username, p.link
            FROM posts p 
            JOIN users u 
            ON p.user_id = u.id
            ORDER BY p.id DESC
            LIMIT 10
            OFFSET $1
            ;", page * 10
        )
            .fetch_all(pool)
            .await?;
    Ok(result)
}

pub async fn read_from_id(post_id: i32, pool: &sqlx::PgPool) -> Result<Post, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Post,
            "SELECT p.title, p.content, p.id, u.username, p.link FROM Posts p JOIN users u ON p.user_id = u.id WHERE p.id=$1;", post_id)
            .fetch_one(pool).await?;
    Ok(result)
}

pub async fn get_posts_from_user(username: String, page: i64, pool: &sqlx::PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Post,
            "SELECT p.title, p.content, p.id, u.username, p.link
            FROM Posts p
            JOIN users u
            ON p.user_id = u.id
            WHERE username=$1
            ORDER BY p.id
            LIMIT 10
            OFFSET $2
            ;", username, page * 10)
            .fetch_all(pool).await?;
    Ok(result)
}

pub async fn check_if_user_can_edit_post(username: String, post_id: i32, pool: &sqlx::PgPool) -> Result<bool, sqlx::Error>  {
    let result = 
        sqlx::query!(
        "SELECT exists (
            SELECT 1 From posts p 
            join users u on p.user_id = u.id
            where u.username=$1 and p.id = $2
        );",
        username,
        post_id)
        .fetch_one(pool)
        .await?;
    Ok(result.exists.unwrap())
}
