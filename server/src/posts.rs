use std::fmt::Display;

use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::{ApiResponse, Object};
use poem_openapi::payload::Json;
use sqlx::Row;

#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct Post {
    pub postid: Option<i32>,
    pub userid: Option<i32>,
    pub title: String,
    pub content: String,
}



impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, TITLE: {}, POSTS: {}", self.postid.unwrap(), self.title, self.content)
    }
}

pub async fn create(title: String, content: String, user_id: i32, pool: &sqlx::PgPool) -> Result<i32, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query("INSERT INTO posts (title, content, userid) VALUES ($1, $2, $3) RETURNING postid;")
            .bind(user_id)
            .bind(title)
            .bind(content)
            .fetch_one(pool)
            .await?;
    Ok(result.get("postid"))
}

pub async fn update(title: String, content: String, post_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!("UPDATE posts set title=$1, content=$2 WHERE postid=$3;", title, content, post_id)
            .execute(pool)
            .await?;
    Ok(result)
}

pub async fn delete(post_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!("DELETE FROM posts WHERE postid=$1;", post_id)
            .execute(pool)
            .await?;
    Ok(result)
}

pub async fn read_all_posts(pool: &sqlx::PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Post, "SELECT * FROM posts;")
            .fetch_all(pool)
            .await?;
    Ok(result)
}

pub async fn read_from_id(post_id: i32, pool: &sqlx::PgPool) -> Result<Post, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Post,
            "SELECT * FROM Posts WHERE postid=$1;", post_id)
            .fetch_one(pool).await?;
    Ok(result)
}