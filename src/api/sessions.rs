
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Local;

#[derive(Default, Debug, FromRow, Serialize, Deserialize)]
pub struct UserSession {
    pub id: i32,
    pub user_id: i32,
    pub session_id: String,
    pub expires_at: DateTime<Local>,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
pub struct UserProfile {
    pub email: String,
    pub username: String,
}

pub async fn get(session_id: String, pool: &sqlx::PgPool) -> Result<UserProfile, sqlx::Error> {
    let result: UserProfile = sqlx::query_as!(UserProfile, 
        "Select u.email, u.username from sessions s
        left join users u
        on s.user_id = u.id
        where s.session_id=$1
        limit 1;",
        session_id
    ).fetch_one(pool).await?;
    Ok(result)
}