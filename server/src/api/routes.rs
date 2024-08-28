use poem_openapi::{payload::{PlainText, Json}, ApiResponse};
use crate::api::users::User;
use crate::api::posts::Post;


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

#[derive(ApiResponse)]
pub enum CreatePostReponse {
    #[oai(status = 200)]
    Ok(Info),

    #[oai(status = 400)]
    InvalidRequest(Info),

    #[oai(status = 404)]
    NotFound,
}



#[derive(poem_openapi::ResponseContent)]
pub enum Info {
    Info(PlainText<String>)
}