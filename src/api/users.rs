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
use sqlx::{Row, Pool, Postgres};
use crate::api::utils::{ApiAuthResponse, check_user_creds};
use crate::api::posts::{self, Post, PostsTemplate};
use crate::api::routes;

#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Object, Clone)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "userid: {}, username: {}, email: {}", self.id, self.username, self.email)
    }
}

pub struct UsersApi;

#[OpenApi(prefix_path = "/api")]
impl UsersApi {
    // Responds with the user's email and username from discord
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

    // get one user
    #[oai(path="/user/:maybe_username", method="get")]
    async fn get_user_from_id(
        &self,
        Path(username): Path<String>,
        Data(pool): Data<&Pool<Postgres>>,
    ) -> PlainText<String> {
        
        let user = read_username(username, &pool).await; 
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
        let user_id = create(req.username.clone(), req.email.clone(), &pool).await;
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
        let result = update(username.unwrap(), email.unwrap(), user_id, &pool).await;        
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
    ) -> routes::UserDeleteResponse {
        let result = delete(user_id, &pool).await;
        match result {
            Ok(_) => routes::UserDeleteResponse::Ok(routes::Info::Info(PlainText(result.unwrap().username))),
            Err(_e) => routes::UserDeleteResponse::NotFound
        }
    }
}

pub async fn create(username: String, email: String, pool: &sqlx::PgPool) -> Result<i32, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query("INSERT INTO users (username, email) VALUES ($1, $2) RETURNING userid;")
            .bind(username)
            .bind(email)
            .fetch_one(pool)
            .await?;
    Ok(result.get("userid"))
}

pub async fn update(username: String, email: String, user_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result: PgQueryResult = 
            sqlx::query!("UPDATE users set username=$1, email=$2 WHERE id=$3;", username, email, user_id)
            .execute(pool)
            .await?;
    Ok(result)
}

pub async fn delete(user_id: i32, pool: &sqlx::PgPool) -> Result<User, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query_as!(User, "DELETE FROM users WHERE id=$1 RETURNING *;", user_id)
            .fetch_one(pool)
            .await?;
    Ok(result)
}

pub async fn read_all(pool: &sqlx::PgPool) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(User, "SELECT * FROM users;")
            .fetch_all(pool)
            .await?;
    Ok(result)
}

pub async fn read_username(username: String, pool: &sqlx::PgPool) -> Result<User, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(User, "SELECT * FROM users WHERE username=$1;", username)
            .fetch_one(pool).await?;
    
    Ok(result)
}


