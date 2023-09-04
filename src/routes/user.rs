use crate::{error::EchoError, structs::{AppState, TokenClaims}};
use actix_web::{
    post,
    web::Data,
    HttpResponse,
    HttpMessage,
    dev::ServiceRequest
};
use std::result::Result;
use uuid::Uuid;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use sha2::Sha256;
use actix_web_httpauth::extractors::{
    bearer::{self, BearerAuth},
    AuthenticationError,
};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use jwt::SignWithKey;


pub async fn token_validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    // grab secret and create Hmac key with it 
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET env var must be set!");
    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret.as_bytes()).unwrap();

    // Verify the token
    let token_string = credentials.token();
    let claims: Result<TokenClaims, &str> = token_string
        .verify_with_key(&key)
        .map_err(|_| "Invalid token!");

    match claims {
        Ok(val) => {
            req.extensions_mut().insert(val);
            Ok(req)
        }
        Err(_) => {
            let config = req.app_data::<bearer::Config>().cloned().unwrap_or_default().scope("");
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SignUpPayload {
    hash: String
}
// User Sign Up
#[post("sign-up")]
/// Example endpoint to echo back a payload from a POST request
pub async fn sign_up(
    state: Data<AppState>,
    payload: String,
) -> Result<HttpResponse, EchoError> {
    let username = payload.split("=").collect::<Vec<&str>>()[1];

    // Create user id
    let user_id = Uuid::new_v4();

    // Generate hash for user to store
    let mut hasher = DefaultHasher::new();
    user_id.hash(&mut hasher);
    let response = SignUpPayload {
        hash: format!("{:#01x}", hasher.finish())
    };

    // Create row in db user table
    match sqlx::query(
        "INSERT INTO users 
            (id, username, hash) 
        VALUES
            ($1, $2, $3)",
    )
    .bind(&user_id)
    .bind(&username)
    .bind(&response.hash)
    .execute(&state.db_pool)
    .await
    {   
        Ok(_) => {
            let sign_up_html = format!("<div class='bg-secondary shadow sm:rounded-lg p-6 mx-auto mt-10' style='width: 50%;'>
        <h3 class='px-4 text-base font-semibold leading-6 text-white text-center'>Save Your Private Key:</h3>
        <div class='mt-2 text-sm text-gray-300 text-center'>
          <p>If on web save it as a file (recommended to use encrypted filesystem)</p>
          <p>If on mobile save it in notes</p>
        </div>
        <div
          hx-boost='true'
          class='flex flex-col items-center w-full'>
          <div class='flex pt-3 mb-5 w-full mx-auto items-center'>
            <label class='inline-block text-xs font-medium text-white ml-auto align-middle'>
              Private Key: 
            </label>
            <input
              id='hash-key'
              class='inline-block w-1/3 rounded-md ml-1 mr-auto border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6 text-center'
              value='{}'
            ></input>
          </div>
          <a
            href='/sign-in'
            class='mt-10 w-12 items-center justify-center rounded-md bg-accent px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 sm:ml-3 sm:mt-0 sm:w-auto'
          >
            I Saved It
          </a>
        </div>", response.hash);
            Ok(HttpResponse::Ok().body(sign_up_html))
        }
        Err(e) => {
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().body(""))
        }
    }
}


#[derive(Serialize, Deserialize, Debug, FromRow)]
struct SignInResponse {
    id: Uuid,
    username: String,
}

// User Sign Up
#[post("sign-in")]
/// Example endpoint to echo back a payload from a POST request
pub async fn sign_in(
    state: Data<AppState>,
    payload: String,
) -> Result<HttpResponse, EchoError> {
    // Consume path (hash) ownership
    let hash = payload.split("=").collect::<Vec<&str>>()[1];

    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
        .expect("JWT_SECRET env var must be set!")
        .as_bytes()
    ).unwrap();

    // Query db for user with hash
    match sqlx::query_as::<_, SignInResponse>(
        "SELECT id, username FROM users 
        WHERE hash = $1",
    )
    .bind(&hash)
    .fetch_one(&state.db_pool)
    .await
    {   
        Ok(row) => {
            let claims = TokenClaims { id: row.id };
            let token_str = claims.sign_with_key(&jwt_secret).unwrap();

            Ok(HttpResponse::Ok().insert_header(("Set-Cookie", token_str.clone())).insert_header(("HX-Location", "http://localhost:8080")).body(token_str))
        }
        Err(e) => {
            println!("{:?}", e);
            Ok(HttpResponse::InternalServerError().body(""))
        }
    }
}

