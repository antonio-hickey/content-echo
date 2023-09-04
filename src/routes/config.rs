use crate::routes;
use actix_web::web;
use actix_web_httpauth::
    middleware::HttpAuthentication;
use routes::user::token_validator;


/// Configures all the api routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let bearer_middleware = HttpAuthentication::bearer(token_validator);

    cfg.service( 
        web::scope("")
            // Website routes
            .service(routes::web::get_index)
            .service(routes::web::get_css)
            .service(routes::web::get_htmx_js)
            .service(routes::web::get_logo)
            .service(routes::web::get_sign_up_html)
            .service(routes::web::get_sign_in_html)
            .service(routes::web::get_saved_feed_html)

            // API Routes
            .service(
                // User routes
                web::scope("/user")
                    .service(routes::user::sign_up)
                    .service(routes::user::sign_in),
            )
            .service(
                // Post routes
                web::scope("/posts")
                    .service(routes::posts::get_feed)
            )
            .service(
                // Post routes
                web::scope("/auth-actions")
                    .wrap(bearer_middleware)
                    .service(routes::posts::get_saved_posts)
                    .service(routes::posts::save) 
            )
    );
}
