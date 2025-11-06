use poem_openapi::OpenApi;

use poem_openapi::payload::{Html};
use poem_openapi::param::Query;
use poem::session::Session;
use poem::web::Data;
use poem::session::CookieConfig;

use sqlx::prelude::FromRow;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Local;
use sqlx::{Pool, Postgres};
use std::env;

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl, AuthType
};
use reqwest::Response;
use oauth2::basic::BasicClient;

use crate::api::utils::{ApiAuthResponse, SID};


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



pub struct AuthApi;

#[OpenApi(prefix_path = "/api")]
impl AuthApi {
    // Handles the response from discord oauth
    // added the user to the session database and redirects to their profile
    #[oai(path="/auth/discord/redirect", method="get")]
    async fn discord_auth(
        &self,
        Query(code): Query<String>,
        Data(pool): Data<&Pool<Postgres>>,
        // Data(middle): Data<&BasicClient>,
        //cookie_jar: &CookieJar
        session: &Session,
    ) -> ApiAuthResponse {
        let client_id = env::var("CLIENT_ID").unwrap();
        let client_secret = env::var("CLIENT_SECRET").unwrap();
        // let redirect_url = "http://localhost:3000/api/auth/discord/redirect".to_string();
        let redirect_url = env::var("REDIRECT_URL").unwrap();
        
        let auth_url = AuthUrl::new("https://discord.com/oauth2/authorize".to_string())
            .expect("Wrong auth endpoint");
        let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
            .expect("Wrong token url");
        
        let client = BasicClient::new(ClientId::new(client_id))
            .set_auth_type(AuthType::RequestBody)
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

        // let http_c: BasicClient = middle.clone();
            // Following redirects opens the client up to SSRF vulnerabilities.
        // let http_client = reqwest::blocking::ClientBuilder::new()
        //     .redirect(reqwest::redirect::Policy::none())
        //     .build()
        //     .expect("Client should build");

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
        println!("{code}");
        let request = client.exchange_code(AuthorizationCode::new(code.clone()));
        println!("{:?}", request);
        let token = match request.request_async(&http_client).await {
            Ok(token) => token,
            Err(e) => return ApiAuthResponse::InvalidRequest(Html(e.to_string() + "<p>something went wrong trying to exchange the code for a token<p>"))
        };
        println!("done");
        
        
        let ctx = reqwest::Client::new();
        let response: Response = ctx.get("https://discord.com/api/v10/users/@me")
            .bearer_auth(token.access_token().secret().to_owned())
            .send().await.unwrap();
        let profile = response.json::<UserProfile>().await.unwrap();
        let Some(secs) = token.expires_in() else {

            return ApiAuthResponse::InvalidRequest(Html("<p>could not find token expiration<p>".to_string()))
        };

        let secs = secs.as_secs();

        let max_age = Local::now() + chrono::Duration::try_seconds(secs.try_into().unwrap()).unwrap();
        
        // creates cookie
        let _cookie = CookieConfig::default()
            .name(SID)
            .domain("localhost")
            .max_age(core::time::Duration::from_secs(secs));
        
        session.set(SID, token.access_token().secret());
        
        // cookie.set_cookie_value(cookie_jar, token.access_token().secret()); // adds it to the cookie jar
        // println!("{}", cookie_jar.get(SID).unwrap().to_string());

        // println!("{}", cookie.get_cookie_value(cookie_jar).unwrap());

        let _userid = sqlx::query!("insert into users (email, username) values ($1, $2) on conflict (username) do nothing;",
            profile.email, profile.username).execute(pool).await.unwrap();


        let _test = sqlx::query!(
                "INSERT INTO sessions (user_id, session_id, expires_at) VALUES (
                (SELECT ID FROM USERS WHERE email = $1 LIMIT 1),
                 $2, $3)
                ON CONFLICT (user_id) DO UPDATE SET
                session_id = excluded.session_id,
                expires_at = excluded.expires_at;",
                profile.email,
                token.access_token().secret().to_owned(),
                max_age
            )
            .execute(pool)
            .await.unwrap();

        ApiAuthResponse::Redirect("/me".to_string())
    }
}
