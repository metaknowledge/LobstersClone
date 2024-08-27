use std::fmt::Display;

use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use sqlx::Row;
#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct User {
    pub userid: i32,
    pub username: String,
    pub email: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "userid: {}, username: {}, email: {}", self.userid, self.username, self.email)
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
            sqlx::query!("UPDATE users set username=$1, email=$2 WHERE userid=$3;", username, email, user_id)
            .execute(pool)
            .await?;
    Ok(result)
}

pub async fn delete(user_id: i32, pool: &sqlx::PgPool) -> Result<User, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query_as!(User, "DELETE FROM users WHERE userid=$1 RETURNING *;", user_id)
            .fetch_one(pool)
            .await?;
    Ok(result)
}

pub async fn read_username(username: String, pool: &sqlx::PgPool) -> Result<User, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(User, "SELECT * FROM users WHERE username=$1;", username)
            .fetch_one(pool).await?;
    
    Ok(result)
}