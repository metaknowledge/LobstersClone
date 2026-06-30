use std::fmt::Display;
use askama::Template;
use poem_openapi::OpenApi;

use poem_openapi::payload::Html; 
use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use poem_openapi::param::{Path, Query};
use poem::web::Data;
use sqlx::{Pool, Postgres};
use crate::api::utils;

#[derive(Default, Debug, FromRow, Serialize, Deserialize, Object)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32, 
    pub username: String,
    pub comment: String,
}

#[derive(Template)]
#[template(path = "comments.html")]
pub struct CommentsTemplate {
    pub comments: Vec<Comment>,
}

#[derive(Template)]
#[template(path = "comment.html")]
struct CommentTemplate {
    pub comment: Comment,
    pub username: String
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Comment: {}, made on post_id: {}, username: {}", self.comment, self.post_id, self.username)
    }
}

pub struct CommentsApi; 


#[OpenApi(prefix_path = "/api")]
impl CommentsApi {
    // queries the database based on the page number, pages are 10 posts long
    #[oai(path="/comments/:post_id", method="get")]
    async fn get_comments(
        &self,
        Path(post_id): Path<Option<i32>>,
        pool: Data<&Pool<Postgres>>
    ) -> utils::ApiAuthResponse {
        match post_id {
            None => return utils::ApiAuthResponse::InvalidRequest(Html("no post specified".to_string())),
            Some(post_id) => {
                let comments: Vec<Comment> = get_post_comments(post_id, &pool).await.unwrap();
                if comments.len() == 0 {
                    return utils::ApiAuthResponse::Ok(Html("No Comments".to_string()))
                }
                let html = CommentsTemplate {comments: comments}
                    .render()
                    .map_err(poem::error::InternalServerError)
                    .unwrap();
                return utils::ApiAuthResponse::Ok(Html(html))

            }
        }
    }
}

// pub async fn create(comment: String, post_id: String, commenter_username: String, pool: &sqlx::PgPool) -> Result<i32, Box<dyn std::error::Error>> {
//     let result = 
//             sqlx::query!(
//                 "INSERT INTO comments (comment, post_id, user_id)
//                 VALUES ($1, $2, (SELECT u.id FROM users u WHERE username=$3))
//                 RETURNING id;",
//                 comment,
//                 post_id,
//                 commenter_username
//             )
//             .fetch_one(pool)
//             .await?;
//     Ok(result.id)
// }

// pub async fn update(title: String, content: String, post_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
//     let result = 
//             sqlx::query_as!(Post, "UPDATE posts set title=$1, content=$2 WHERE id=$3;", title, content, post_id)
//             .execute(pool)
//             .await?;
//     Ok(result)
// }

pub async fn delete(comment_id: i32, pool: &sqlx::PgPool) -> Result<PgQueryResult, Box<dyn std::error::Error>> {
    let result = 
            sqlx::query!("DELETE FROM posts WHERE id=$1;", comment_id)
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
            "SELECT c.id, c.post_id, c.comment, u.username
            FROM comments c
            JOIN users u ON u.id=c.user_id
            WHERE c.post_id = $1
            ;", post_id)
            .fetch_all(pool)
            .await?;
    Ok(result)
}


pub async fn get_comment_from_id(comment_id: i32, pool: &sqlx::PgPool) -> Result<Comment, Box<dyn std::error::Error>> {
    let result = 
        sqlx::query_as!(Comment,
            "SELECT c.id, c.post_id, c.comment, u.username
            FROM comments c
            JOIN users u ON u.id=c.user_id
            WHERE c.id=$1;", comment_id)
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
