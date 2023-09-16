use crate::{error::EchoError, structs::AppState};
use actix_web::{
    get,
    web::Data,
};
use actix_files::NamedFile;


#[get("/")]
pub async fn get_index(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/src/index.html").unwrap())
}

#[get("/saved")]
pub async fn get_saved_feed_html(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/src/saved-feed.html").unwrap())
}

#[get("/dist/output.css")]
pub async fn get_css(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/dist/output.css").unwrap())
}

#[get("/htmx.min.js")]
pub async fn get_htmx_js(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/src/htmx.min.js").unwrap())
}

#[get("/assets/logo.png")]
pub async fn get_logo(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/assets/logo.png").unwrap())
}

#[get("/assets/favicon.ico")]
pub async fn get_favicon(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/assets/favicon.ico").unwrap())
}

#[get("/sign-up")]
pub async fn get_sign_up_html(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/src/sign-up.html").unwrap())
}

#[get("/sign-in")]
pub async fn get_sign_in_html(
    _state: Data<AppState>,
) -> Result<NamedFile, EchoError> {
    Ok(NamedFile::open("../../src/website/src/sign-in.html").unwrap())
}


