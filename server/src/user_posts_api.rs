
use poem::web::Data;
use poem::Request;
use poem_openapi::param::{Path, Query};
use poem_openapi::Object;
use poem_openapi::{payload::PlainText, OpenApi};
use sqlx::{Pool, Postgres};
use poem_openapi::payload::{Html, Json};
use crate::users;
use crate::posts::{self, Post};
use crate::routes::{PostApiResponse, UserApiResponse, UserDeleteResponse, Info};
pub struct PostsApi;

#[derive(Object, Clone)]
pub struct CreatePost {
    pub user_id: String,
    pub title: String,
    pub content: String,
}



#[OpenApi]
impl PostsApi {
    #[oai(path="/", method="get")]
    async fn get_site(&self) -> Html<String> {
        // let contents = fs::read_to_string("../frontend/index.html")
            // .expect("Should have been able to read the file");
        let contents = tokio::fs::read_to_string("../frontend/index.html").await.expect("should have been able to read the file");
        Html(contents)
    }


    // get all posts
    #[oai(path="/posts/all/", method="get")]
    async fn get_all_posts(
        &self,
        pool: Data<&Pool<Postgres>>
    ) -> PostApiResponse {
        let posts = posts::read_all_posts(&pool).await.unwrap();        
        PostApiResponse::Ok(Json(posts))
    }

    #[oai(path="/html/posts/all/", method="get")]
    async fn get_all_posts_as_html(
        &self,
        pool: Data<&Pool<Postgres>>
    ) -> Html<String> {
        let posts = posts::read_all_posts(&pool).await.unwrap();  
        let html = Self::into_html(posts);
        // PostApiResponse::Ok(Json(posts))
        Html(html)
    }

    fn into_html(posts: Vec<Post>) -> String {
        let mut html: String = String::from("<tr>");

        html = posts.iter().map(|post| 
            "<tr>".to_string() + 
            &format!("<td>{}</td>", post.title) + 
            &format!("<td>{}</td>", post.username) +
            &format!("<td>{}</td>", post.content) +
            &format!("<td>{}</td>", post.postid) +
            "</tr>"
        ).collect::<String>();

        // for post in posts {
        //     let title = format!("<td>{}</td>", post.title);
        //     let username = format!("<td>{}</td>", post.postid);
        //     let content = format!("<td>{}</td>", post.content);
        //     html = html + &title;
        // }
        return html;
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
    ) -> PlainText<String> {
        let title = create_post.title.clone();
        let content = create_post.content.clone();
        let user_id = create_post.user_id.clone().parse::<i32>().unwrap();
        println!("{:?}, {:?}, {:?}", title, content, user_id);
        let result = posts::create(title, content, user_id, &pool).await;
        match result {
            Ok(postid) => PlainText(format!("{}", postid)),
            Err(_) => PlainText("An error has occured".to_string())
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
        Query(title): Query<Option<String>>,
        Query(content): Query<Option<String>>,
    ) -> PlainText<String> {
        let _ = posts::update(title.unwrap(), content.unwrap(), post_id, &pool).await;        
        PlainText("updated".to_string())  
    }

    // get all users
    #[oai(path="/users/all/", method="get")]
    async fn get_all_users(
        &self,
        pool: Data<&Pool<Postgres>>
    ) -> UserApiResponse {
        let users = users::read_all(&pool).await.unwrap();        
        UserApiResponse::Ok(Json(users))
    }

    // get one user
    #[oai(path="/user/:user_id", method="get")]
    async fn get_user_from_id(
        &self,
        Path(user_id): Path<String>,
        pool: Data<&Pool<Postgres>>,
    ) -> PlainText<String> {
        let user = users::read_from_id(user_id.parse::<i32>().unwrap(), &pool).await.unwrap();        
        PlainText(user.to_string())
    }

    // create user
    #[oai(path="/user/", method="post")]
    async fn create_user(
        &self,
        pool: Data<&Pool<Postgres>>,
        Query(username): Query<Option<String>>,
        Query(email): Query<Option<String>>,
    ) -> PlainText<String> {
        let user_id = users::create(username.unwrap(), email.unwrap(), &pool).await;
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

