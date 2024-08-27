use poem::{handler, IntoResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate;

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    pub postid: i32
}

#[handler]
pub fn home() -> impl IntoResponse {
    let home = HomeTemplate;
    HtmlTemplate (home).into_response()
}

#[handler]
pub fn post() -> impl IntoResponse {
    let id = 71;

    let post = PostTemplate{postid: id};
    HtmlTemplate(post).into_response()
}

// cite: https://github.com/nicolasauler/anodized-poem/blob/main/src/main.rs
struct HtmlTemplate<T>(T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template + Send + Sync + 'static,
{
    fn into_response(self) -> poem::Response {
        let body = self.0.render().unwrap();
        poem::Response::builder()
            .content_type("text/html; charset=utf-8")
            .body(body)
    }
}