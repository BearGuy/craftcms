pub mod admin_assets;
pub mod cli;
pub mod config;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod template_utils;
pub mod templates;

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

    let image_route = warp::path!("images" / String).and(warp::get()).and_then({
        let conn = Arc::clone(&conn);
        move |slug| handlers::serve_image(slug, conn.clone())
    });

    // Admin routes from routes module
    let admin_routes = routes::admin_routes(config.clone(), conn.clone());

    let static_files = warp::path("static").and(warp::fs::dir("static"));

    let routes = home_route
        .or(post_detail_route)
        .or(image_route)
        .or(admin_routes)
        .or(static_files);

    let routes = routes.recover(handle_rejection);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

pub fn setup_database() -> Result<(), rusqlite::Error> {
    let conn = database::init_db()?;
    database::run_migrations(&conn)?;
    Ok(())
}
