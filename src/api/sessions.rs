use std::fmt::Display;

use chrono::NaiveDateTime;
use sqlx::postgres::PgQueryResult;
use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use sqlx::Row;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Local;


#[derive(Default, Debug, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub session_id: String,
    pub expires_at: DateTime<Local>,
}