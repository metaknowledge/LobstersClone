use poem_openapi::payload::PlainText;
use poem_openapi::{Object, ResponseContent};
use poem_openapi::{payload::Json, ApiResponse};
use crate::users::User;
use crate::posts::Post;


#[derive(ApiResponse)]
pub enum UserApiResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<User>>),

    #[oai(status = 400)]
    InvalidRequest,

    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
pub enum UserDeleteResponse {
    #[oai(status = 200)]
    Ok(Info),

    #[oai(status = 400)]
    InvalidRequest,

    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
pub enum PostApiResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<Post>>),

    #[oai(status = 400)]
    InvalidRequest,

    #[oai(status = 404)]
    NotFound,
}

#[derive(ResponseContent)]
pub enum Info {
    Info(PlainText<String>)
}