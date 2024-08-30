use std::fmt::Display;

use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use sqlx::Row;

#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct Post {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub content: String,
}


impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "USERNAME: {}, TITLE: {}, CONTENT: {}", self.username, self.title, self.content)
    }
}

pub async fn create(title: String, content: String, username: String, pool: &sqlx::PgPool) -> Result<i32, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query(
                "INSERT INTO posts (title, content, user_id)
                    VALUES ($1, $2, (SELECT user_id FROM users WHERE username=$3))
                    RETURNING id;")
            .bind(title)
            .bind(content)
            .bind(username)
            .fetch_one(pool)
            .await?;
    Ok(result.get("postid"))
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
            "SELECT p.title, p.content, p.id, u.username
            FROM posts p 
            JOIN users u 
            ON p.user_id = u.id
            ORDER BY p.id
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
            "SELECT p.title, p.content, p.id, u.username FROM Posts p JOIN users u ON p.user_id = u.id WHERE p.id=$1;", post_id)
            .fetch_one(pool).await?;
    Ok(result)
}

pub async fn get_posts_from_user(username: String, page: i64, pool: &sqlx::PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Post,
            "SELECT p.title, p.content, p.id, u.username
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

pub struct Instrument {
    pub id: i32,
    pub expiry_on: chrono::DateTime<chrono::Utc>,
}