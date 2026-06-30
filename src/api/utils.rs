use poem::session::Session;
use poem_openapi::ApiResponse;
use sqlx::{Pool, Postgres};
use poem_openapi::payload::Html;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2;
use std::env;
use crate::api::sessions;

pub fn build_oauth_client(client_id: String, client_secret: String) {
    // let redirect_url = "http://localhost:3000/api/auth/discord/redirect".to_string();
    let redirect_url = env::var("REDIRECT_URL").expect("couldn't find REDIRECT_URL");
    
    let auth_url = AuthUrl::new("https://discord.com/oauth2/authorize".to_string())
        .expect("Wrong auth endpoint");
    let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
        .expect("Wrong token url");
    
    BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());
}

pub async fn check_user_creds(session: &Session, pool: &Pool<Postgres>) -> Result<sessions::UserProfile, ApiAuthResponse> {
    let session_id = match session.get::<String>(SID) {
        Some(cookie) => cookie,
        // None => return Err(ApiAuthResponse::Redirect("/signup".to_string())),
        None => return Err(ApiAuthResponse::Redirect("/signup".to_string()))
    };
    let res = match sessions::get(session_id, pool).await {
        Ok(result) => result,
        Err(_err) => return Err(ApiAuthResponse::NotAuthorized)
    };
    Ok(res)
}


pub const SID: &str = "sid";


// #[derive(ApiResponse)]
// enum RedirectResponse {
//     #[oai(status = "307")]
//     Redirect(#[oai(header = "Location")] String),
//     #[oai(status = 400)]
//     InvalidRequest(PlainText<String>),
// }

#[derive(ApiResponse)]
pub enum ApiAuthResponse {
    #[oai(status = 200)]
    Ok(Html<String>),
    #[oai(status = 302)]
    Redirect(#[oai(header = "Location")] String),
    #[oai(status = 400)]
    InvalidRequest(Html<String>),
    #[oai(status = 404)]
    NotFound,
    #[oai(status = 401)]
    NotAuthorized,
}

