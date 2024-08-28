use askama::Template;
use poem::web::Data;
use poem_openapi::param::{Path, Query};
use poem_openapi::Object;
use poem_openapi::{payload::PlainText, OpenApi};
use sqlx::{Pool, Postgres};
use poem_openapi::payload::{Html, Json};
use crate::api::users;
use crate::api::posts::{self, Post};
use crate::api::routes::{UserDeleteResponse, Info};

use super::routes::{CreatePostReponse, UserApiResponse};
pub struct PostsApi;

#[derive(Object, Clone)]
pub struct CreatePost {
    pub username: String,
    pub title: String,
    pub content: String,
}

#[derive(Object, Clone, Default)]
pub struct UpdatePost {
    pub title: String,
    pub content: String,
}

#[derive(Object, Clone)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "posts.html")]
struct PostsTemplate {
    pub posts: Vec<Post>,
    pub page: i64,
    pub editable: bool,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    pub post: Post,
    pub editable: bool,
    pub i: i32
}

#[derive(Template)]
#[template(path = "editpost.html")]
struct EditPostTemplate {
    pub postid: i64,
    pub title: String,
    pub content: String,
}

#[OpenApi]
impl PostsApi {
    #[oai(path="/html/posts", method="get")]
    async fn get_paged_html(
        &self,
        Query(page): Query<Option<i64>>,
        pool: Data<&Pool<Postgres>>
    ) -> Html<String> {
        let posts: Vec<Post> = posts::read_page_number(page.unwrap(), &pool).await.unwrap();  
        if posts.len() == 0 {
            return Html("You made it to the bottom".to_string())
        }
        let html = PostsTemplate {posts: posts, page: page.unwrap() + 1, editable: false}
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }

    // get one post
    #[oai(path="/post/:post_id", method="get")]
    async fn get_post_from_id(
        &self,
        Path(post_id): Path<String>,
        pool: Data<&Pool<Postgres>>,
    ) -> PlainText<String> {
        let post = posts::read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        PlainText(post.to_string())
    }

    // create post
    #[oai(path="/post", method="post")]
    async fn post_post(
        &self,        
        pool: Data<&Pool<Postgres>>,
        create_post: Json<CreatePost>
    ) -> CreatePostReponse {
        let title = create_post.title.clone();
        let content = create_post.content.clone();
        let username = create_post.username.clone();
        println!("{}{}{}", title, content, username);
        let result = posts::create(title, content, username, &pool).await;
        match result {
            Ok(postid) => CreatePostReponse::Ok(Info::Info(PlainText(postid.to_string()))),
            Err(err) => CreatePostReponse::InvalidRequest(Info::Info(PlainText(err.to_string() + ": An error has occured")))
        }   
    }

    // delete post
    #[oai(path="/post/:post_id", method="delete")]
    async fn delete_post(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(post_id): Path<i32>,
    ) -> PlainText<String> {
        let _ = posts::delete(post_id, &pool).await;        
        PlainText("deleted".to_string())  
    }

    // update post
    #[oai(path="/post/:post_id", method="put")]
    async fn update_post(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(post_id): Path<i32>,
        update_post: Json<UpdatePost>
    ) -> Html<String> {
        let title = update_post.title.clone();
        let content = update_post.content.clone();
        let result = posts::update(title.clone(), content.clone(), post_id, &pool).await.unwrap();        
        match result.rows_affected() {
            0 => Html("could not update row".to_string()),
            _ => {
                let post = Post {
                    title: title,
                    postid: post_id,
                    username: String::new(),
                    content: content
                };
                let html = PostTemplate {post: post, editable: true, i: 0}
                    .render()
                    .map_err(poem::error::InternalServerError)
                    .unwrap();
                Html(html)
            }
        }
    }
    
    // fn render_html(template: impl Template) -> String {
    //     template.render().map_err(poem::error::InternalServerError("oh no")).unwrap()
    // }

    #[oai(path="/post/:post_id/edit", method="get")]
    async fn edit_post_html(
        &self,
        Path(post_id): Path<String>,
        pool: Data<&Pool<Postgres>>,
    ) -> Html<String> {
        let post = posts::read_from_id(post_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        let html = EditPostTemplate {postid: post_id.parse::<i64>().unwrap(), title: post.title, content: post.content }
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }

    // get one user
    #[oai(path="/user/:maybe_username", method="get")]
    async fn get_user_from_id(
        &self,
        Path(maybe_username): Path<Option<String>>,
        pool: Data<&Pool<Postgres>>,
    ) -> PlainText<String> {
        match maybe_username {
            Some(username) => {
                let user = users::read_username(username, &pool).await; 
                match user {
                Ok(user) => PlainText(user.to_string()),
                Err(error) => PlainText(error.to_string() + ": Error please fix")
                }
                
            },
            None => PlainText("No username provided".to_string())
        }
    }

    // get one user
    #[oai(path="/user/posts/:username", method="get")]
    async fn get_user_posts(
        &self,
        Path(username): Path<String>,
        Query(page_number): Query<Option<i64>>,
        pool: Data<&Pool<Postgres>>,
    ) -> Html<String> {
        let page = match page_number {
            Some(page) => page,
            None => 0
        };
        let posts: Vec<Post> = posts::get_posts_from_user(username, page, &pool).await.unwrap();        
        if posts.len() == 0 {
            return Html("You made it to the bottom".to_string())
        }
        let html = PostsTemplate {posts: posts, page: page + 1, editable: true}
            .render()
            .map_err(poem::error::InternalServerError)
            .unwrap();
        Html(html)
    }

    // create user
    #[oai(path="/user", method="post")]
    async fn create_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        req: Json<CreateUser>
    ) -> PlainText<String> {
        let user_id = users::create(req.username.clone(), req.email.clone(), &pool).await;
        PlainText(user_id.unwrap().to_string())
    }

    // update user
    #[oai(path="/user/:user_id", method="put")]
    async fn update_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(user_id): Path<i32>,
        Query(username): Query<Option<String>>,
        Query(email): Query<Option<String>>,
    ) -> PlainText<String> {
        let result = users::update(username.unwrap(), email.unwrap(), user_id, &pool).await;        
        match result {
            Ok(_) => PlainText("updated".to_string()),  
            Err(_) => PlainText("could not update".to_string()),
        }
    }

    // delete user
    #[oai(path="/user/:user_id", method="delete")]
    async fn delete_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        Path(user_id): Path<i32>,
    ) -> UserDeleteResponse {
        let result = users::delete(user_id, &pool).await;
        match result {
            Ok(_) => UserDeleteResponse::Ok(Info::Info(PlainText(result.unwrap().username))),
            Err(_e) => UserDeleteResponse::NotFound
        }
    }
}

pub fn get_service() -> poem_openapi::OpenApiService<PostsApi, ()> {
    let api_service =
        poem_openapi::OpenApiService::new(PostsApi, "Hello World", "1.0");
    api_service
    
}
