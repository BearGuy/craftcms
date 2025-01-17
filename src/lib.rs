pub mod admin_assets;
pub mod cli;
pub mod commands;
pub mod config;
pub mod database;
pub mod files;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod template_utils;
pub mod templates;

use files::ImageFileManager;
use models::{CustomError, RedirectToLogin};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use warp::http::{header::LOCATION, Response, StatusCode};
use warp::Filter;

fn with_db(
    db: Arc<Mutex<Connection>>,
) -> impl Filter<Extract = (Arc<Mutex<Connection>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_config(
    config: Arc<config::Config>,
) -> impl Filter<Extract = (Arc<config::Config>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

pub async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.is_not_found() {
        let response = Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".to_string())
            .unwrap();
        Ok(response)
    } else if err.find::<RedirectToLogin>().is_some() {
        let response = Response::builder()
            .status(StatusCode::FOUND)
            .header(LOCATION, "/admin/login".to_string())
            .body("".to_string())
            .unwrap();
        Ok(response)
    } else if let Some(e) = err.find::<CustomError>() {
        let response = Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(e.message.clone())
            .unwrap();
        Ok(response)
    } else if let Some(_) = err.find::<warp::reject::PayloadTooLarge>() {
        println!("Payload too large");
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("File too large. Maximum size is 5MB.".to_string())
            .unwrap();
        Ok(response)
    } else {
        let response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Internal Server Error".to_string())
            .unwrap();
        Ok(response)
    }
}

pub async fn run_server() {
    // Load configuration
    let config = Arc::new(config::Config::load().expect("Failed to load configuration"));

    // Initialize database connection
    let conn = Arc::new(Mutex::new(
        database::init_db().expect("Failed to initialize database"),
    ));

    // Initialize file manager
    let file_manager = Arc::new(ImageFileManager::new("data/images"));

    // Define routes
    let home_route = warp::path::end()
        .and(warp::get())
        .and(with_config(config.clone()))
        .and(with_db(conn.clone()))
        .and_then(handlers::home_handler);

    let post_detail_route = warp::path(config.routes.detail_path.clone())
        .and(warp::path::param())
        .and(warp::get())
        .and(with_config(config.clone()))
        .and(with_db(conn.clone()))
        .and_then(handlers::post_detail_handler);

    // Admin routes from routes module
    let admin_routes = routes::admin_routes(config.clone(), conn.clone(), file_manager.clone());

    let image_routes = warp::path("images").and(warp::fs::dir("data/images"));

    let static_files = warp::path("static").and(warp::fs::dir("static"));

    let routes = home_route
        .or(post_detail_route)
        .or(image_routes)
        .or(admin_routes)
        .or(static_files);

    let routes = routes.recover(handle_rejection);

    // Start the server
    warp::serve(routes)
        .run((config.server.get_ip_addr(), config.server.port))
        .await;
}

pub fn setup_database() -> Result<(), rusqlite::Error> {
    let conn = database::init_db()?;
    database::run_migrations(&conn)?;
    Ok(())
}
