use std::fmt::Display;

use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;

#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32, 
    pub user_id: i32,
    pub comment: String,
}

impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Comment: {}, post_id: {}", self.comment, self.post_id)
    }
}

pub async fn create(comment: String, post_id: String, commenter_username: String, pool: &sqlx::PgPool) -> Result<i32, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!(
                "INSERT INTO comments (comment, post_id, user_id)
                VALUES ($1, $2, (SELECT u.id FROM users u WHERE username=$3))
                RETURNING id;",
                comment,
                post_id,
                commenter_username
            )
            .fetch_one(pool)
            .await?;
    Ok(result.id)
}

// pub async fn update(title: String, content: String, post_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
//     let result = 
//             sqlx::query_as!(Post, "UPDATE posts set title=$1, content=$2 WHERE id=$3;", title, content, post_id)
//             .execute(pool)
//             .await?;
//     Ok(result)
// }

pub async fn delete(comment_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!("DELETE FROM posts WHERE id=$1;", post_id)
            .execute(pool)
            .await?;
    Ok(result)
}

// pub async fn read_page_number(page: i64, pool: &sqlx::PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
//     let result = 
//         sqlx::query_as!(Post,
//             "SELECT p.title, p.content, p.id, u.username, p.link
//             FROM posts p 
//             JOIN users u 
//             ON p.user_id = u.id
//             ORDER BY p.id DESC
//             LIMIT 10
//             OFFSET $1
//             ;", page * 10
//         )
//             .fetch_all(pool)
//             .await?;
//     Ok(result)
// }

pub async fn get_post_comments(post_id: i32, pool: &sqlx::PgPool) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Comment,
            "SELECT *
            FROM comments c
            WHERE c.post_id = $1
            ;", post_id)
            .fetch_all(pool)
            .await?;
    Ok(result)
}


pub async fn get_comment_from_id(comment_id: i32, pool: &sqlx::PgPool) -> Result<Post, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Comment,
            "SELECT * FROM comments p WHERE p.id=$1;", post_id)
            .fetch_one(pool).await?;
    Ok(result)
}

// pub async fn get_posts_from_user(username: String, page: i64, pool: &sqlx::PgPool) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
//     let result = 
//         sqlx::query_as!(Post,
//             "SELECT p.title, p.content, p.id, u.username, p.link
//             FROM Posts p
//             JOIN users u
//             ON p.user_id = u.id
//             WHERE username=$1
//             ORDER BY p.id
//             LIMIT 10
//             OFFSET $2
//             ;", username, page * 10)
//             .fetch_all(pool).await?;
//     Ok(result)
// }

// pub async fn check_if_user_can_edit_post(username: String, post_id: i32, pool: &sqlx::PgPool) -> Result<bool, sqlx::Error>  {
//     let result = 
//         sqlx::query!(
//         "SELECT exists (
//             SELECT 1 From posts p 
//             join users u on p.user_id = u.id
//             where u.username=$1 and p.id = $2
//         );",
//         username,
//         post_id)
//         .fetch_one(pool)
//         .await?;
//     Ok(result.exists.unwrap())
// }
