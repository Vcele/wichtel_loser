mod handlers;
mod models;
mod state;

use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};
use state::AppState;
use std::sync::Arc;
use tera::Tera;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Template parsing error: {}", e);
            std::process::exit(1);
        }
    };

    let app_state = Arc::new(AppState::new());
    let secret_key = Key::generate();

    let bind_addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    
    println!("🎄 Wichtel Loser starting at http://{}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::from(app_state.clone()))
            .service(handlers::index)
            .service(handlers::create_event_page)
            .service(handlers::create_event)
            .service(handlers::join_page)
            .service(handlers::join_event)
            .service(handlers::manage_event)
            .service(handlers::close_event)
            .service(handlers::view_assignment)
            .service(handlers::identify_page)
            .service(handlers::search_participants)
            .service(handlers::confirm_identity)
            .service(actix_files::Files::new("/static", "static").show_files_listing())
    })
    .bind(&bind_addr)?
    .run()
    .await
}
