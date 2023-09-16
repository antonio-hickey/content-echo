use crate::{error::EchoError, structs::{AppState, TokenClaims}};
use actix_web::{
    post, get,
    web::{Data, Json, Path, ReqData},
    HttpResponse, HttpRequest,
};
use sqlx::FromRow;
use std::{result::Result, hash::{Hash, Hasher}, future::{self, Future}, collections::VecDeque, sync::Arc};
use uuid::Uuid;
use std::collections::hash_map::DefaultHasher;
use serde::{Serialize, Deserialize};
use reqwest::{Response, Client};

use futures::future::join_all;
use std::error::Error;


#[derive(Serialize, Deserialize, Hash, Debug, Clone)]
pub struct Post {
    id: String,
    title: String,
    author: String,
    url: String,
    timestamp: String,
}

#[post("/save")]
/// Endpoint for saving posts
pub async fn save(
    state: Data<AppState>,
    payload: Json<Post>,
    req_user: Option<ReqData<TokenClaims>>
) -> Result<HttpResponse, EchoError> {
    // Consume Payload ownership
    match req_user {
        Some(user) => {
            let post_to_save = payload.into_inner();

            // Hash the content of the post struct excluding
            // the user_id, this way we replicate 
            let mut hasher = DefaultHasher::new();
            post_to_save.hash(&mut hasher);
            let post_hash = format!("{:#01x}",hasher.finish());

            // Create row in saved_posts table
            sqlx::query("INSERT INTO posts 
                (hash, post_id, title, url, author, timestamp)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (hash) DO NOTHING
            ")
            .bind(&post_hash)
            .bind(&post_to_save.id.parse::<i64>().unwrap())
            .bind(&post_to_save.title)
            .bind(&post_to_save.url)
            .bind(&post_to_save.author)
            .bind(&post_to_save.timestamp.parse::<i64>().unwrap())
            .execute(&state.db_pool)
            .await?;

            // Add the post_id to users saved posts
            match sqlx::query("UPDATE users
                SET saved_posts = saved_posts || $1
                WHERE id = $2
            ")
            .bind(&post_to_save.id.parse::<i64>().unwrap())
            .bind(&user.id)
            .execute(&state.db_pool)
            .await {
                Ok(_) => {
                    let updated_html = String::from("<button
                      id='save-btn'
                      type='submit'
                      class='rounded-md bg-primary px-3.5 py-2.5 text-sm font-semibold text-white shadow-lg'
                    >Saved</button>");
                    Ok(HttpResponse::Ok().body(updated_html))
                },
                Err(e) => {
                    println!("{:?}", e);
                    Ok(HttpResponse::InternalServerError().body(""))
                },
            }

                }
                None => Ok(HttpResponse::Unauthorized().body(""))
            }

}

#[derive(Serialize, Deserialize, FromRow)]
struct SavedPosts {
    post_id: i64,
    title: String,
    url: String,
    author: String,
    timestamp: i32,
} impl SavedPosts {
    fn into_post(self) -> Post {
        Post {
            id: self.post_id.to_string(),
            title: self.title,
            author: self.author,
            url: self.url,
            timestamp: self.timestamp.to_string(),
        }
    }
}

// User Get Saved Posts
#[post("saved")]
/// Endpoint for saving posts
pub async fn get_saved_posts(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
) -> Result<HttpResponse, EchoError> {
    // Consume path value ownership
    match req_user {
        Some(user) => {
            // Query all saved posts a user has
            match sqlx::query_as::<_, SavedPosts>("SELECT post_id, title, url, author, timestamp 
                FROM posts WHERE post_id = ANY(SELECT unnest(saved_posts) FROM users WHERE id = $1)
            ")
            .bind(&user.id)
            .fetch_all(&state.db_pool)
            .await {
                Ok(saved_posts) => {
                    let mut content_cards_html: Vec<String> = Vec::new();
                    for saved_post in saved_posts.into_iter() {
                        let post = saved_post.into_post();
                        content_cards_html.push(create_post_html_card(&post));
                    }

                    Ok(HttpResponse::Ok().body(content_cards_html.concat()))
                }
                Err(e) => {
                    println!("{:?}", e);
                    Ok(HttpResponse::InternalServerError().body(""))
                }
            }
        },
        None => Ok(HttpResponse::Unauthorized().body(""))
    }
}


async fn get_post(id: i64, client: Arc<Client>) -> Post {
    client.get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id))
        .send()
        .await.unwrap()
        .json::<HnPost>()
        .await.unwrap()
        .into_post()
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone)]
struct HnPost {
    by: String,
    descendants: i32,
    id: i64,
    kids: Vec<i64>,
    score: i32,
    time: i64,
    title: String,
    #[serde(rename = "type")]
    post_type: String,
    url: Option<String>,
} impl HnPost {
    fn into_post(self) -> Post {
        Post {
            author: self.by,
            id: self.id.to_string(),
            title: self.title,
            url: self.url.unwrap_or("".to_string()),
            timestamp: self.time.to_string(),
        }
    }
}

fn create_post_html_card(post: &Post) -> String {
    format!("<li 
            hx-boost='true'
            key={} 
            class=\"overflow-hidden bg-secondary rounded-xl border border-gray-200 max-h-44\"
        >
        <a href='{}'>
        <div
            class=\"w-full group relative cursor-pointer overflow-hidden bg-secondary px-6 pt-1 shadow-xl ring-1 ring-gray-900/5 transition-all duration-300 hover:-translate-y-1 hover:shadow-2xl sm:mx-auto sm:rounded-lg sm:px-10\"
        >
            <span class=\"absolute inset-x-0 top-0 h-6 w-full bg-accent transition-all duration-300 group-hover:scale-[100]\"></span>
            <div class=\"relative z-10 mx-auto max-w-md\">
                <div
                    class=\"space-y-1 pt-5 text-base leading-7 text-gray-600 transition-all duration-300 group-hover:text-white/90\"
                >
                    <h3 class='truncate text-xl font-extrabold text-white'>{}</h3>
                    <p class=\"mt-1 truncate text-sm text-gray-100\">Author: {}</p>
                </div>
                <form
                    class='save-post-form flex w-full'
                    hx-post='/auth-actions/save'
                    hx-trigger='submit'
                    hx-target='#save-btn-{}'
                    hx-swap='outerHTML'
                    hx-ext='json-enc'
                    hx-indicator='#spinner'
                >
                    <input id='id' name='id' class='invisible hidden' value='{}'></input>
                    <input id='title' name='title' class='invisible hidden' value='{}'></input>
                    <input id='author' name='author' class='invisible hidden' value='{}'></input>
                    <input id='url' name='url' class='invisible hidden' value='{}'></input>
                    <input id='timestamp' name='timestamp' class='invisible hidden' value='{}'></input>
                    <button
                      id='save-btn-{}'
                      type='submit'
                      class='save-post-btn rounded-md mx-auto my-5 bg-primary px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-white/50 transition-all duration-300 group-hover:bg-secondary'
                    >
                        <span id='save-post-text'>Save Post</span>
                        <div id='spinner' style='display: none;'>
                            <svg aria-hidden='true' role='status' class='inline w-4 h-4 mr-3 text-accent text-center animate-spin' viewBox='0 0 100 101' fill='none' xmlns='http://www.w3.org/2000/svg'>
                                <path d='M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z' fill='#E5E7EB'/>
                                <path d='M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z' fill='currentColor'/>
                            </svg>
                        </div>
                    </button>
                </form>
            </div>
        </div>
        </a>
      </li>", &post.id, &post.url, &post.title, &post.author, &post.id, &post.id, post.title, post.author, post.url, post.timestamp, post.id)
}


#[get("/feed")]
pub async fn get_feed(
    state: Data<AppState>,
) -> Result<HttpResponse, EchoError> {
    let client = Arc::new(Client::new());

    let post_ids = reqwest::get("https://hacker-news.firebaseio.com/v0/beststories.json")
        .await.unwrap()
        .json::<Vec<i64>>()
        .await.unwrap();

    // Use a VecDeque to handle asynchronous requests
    let mut request_queue: VecDeque<_> = post_ids
        .iter()
        .map(|id| get_post(id.to_owned(), client.clone()))
        .collect();

    // Create a vector to store the join handles for each request
    let mut join_handles = Vec::new();

    // Process requests concurrently using tokio::join_all
    while let Some(request) = request_queue.pop_front() {
        let handle = tokio::spawn(request);
        join_handles.push(handle);
    }

    // Wait for all requests to complete
    let mut feed: Vec<Post> = Vec::new();
    let mut content_cards_html: Vec<String> = Vec::new();
    let _ = tokio::join!(async {
        for handle in join_handles {
            let post = handle.await.expect("Failed to join a task");
            if !post.title.contains("HN: ") {
                content_cards_html.push(create_post_html_card(&post));
                feed.push(post);
            }
        }
    });

    // Concatenate the HTML cards into one string
    Ok(HttpResponse::Ok().body(content_cards_html.concat()))
}

