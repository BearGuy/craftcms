use crate::config::Config;
use crate::{handlers::*, middleware::with_auth, with_config, with_db};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use warp::Filter;

use crate::admin_assets::AdminAssets;

pub fn admin_routes(
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let admin_base = warp::path("admin");

    // Login route - no auth required
    let admin_login_page = admin_base
        .and(warp::path("login"))
        .and(warp::get())
        .and(with_config(config.clone()))
        .and_then(admin_login_page_handler);

    let admin_login = admin_base
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(conn.clone()))
        .and_then(admin_login_handler);

    let admin_logout = admin_base
        .and(warp::path("logout"))
        .and(warp::post())
        .and(warp::cookie::optional("session"))
        .and(with_db(conn.clone()))
        .and_then(admin_logout_handler);

    // Main admin page
    let admin_page = admin_base
        .and(warp::path::end())
        .and(with_auth(conn.clone()))
        .and(with_config(config.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_page_handler);

    // New image page
    let admin_new = admin_base
        .and(warp::path("new"))
        .and(with_auth(conn.clone()))
        .and(with_config(config.clone()))
        .and_then(admin_new_image_handler);

    // Create image endpoint
    let admin_create = admin_base
        .and(warp::path("create"))
        .and(with_auth(conn.clone()))
        .and(warp::multipart::form())
        .and(with_db(conn.clone()))
        .and_then(admin_create_image_handler);

    // Edit image page
    let admin_edit = admin_base
        .and(warp::path("edit"))
        .and(warp::path::param())
        .and(with_auth(conn.clone()))
        .and(with_config(config.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_edit_image_handler);

    // Update image endpoint
    let admin_update = admin_base
        .and(warp::path("update"))
        .and(warp::path::param())
        .and(with_auth(conn.clone()))
        .and(warp::multipart::form())
        .and(with_db(conn.clone()))
        .and_then(admin_update_image_handler);

    // Delete image endpoint
    let admin_delete = admin_base
        .and(warp::path("delete"))
        .and(warp::path::param())
        .and(with_auth(conn.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_delete_image_handler);

    let admin_assets = warp::path("admin")
        .and(warp::path("assets"))
        .and(warp_embed::embed(&AdminAssets));

    // Combine all routes
    admin_assets
        .or(admin_login)
        .or(admin_logout)
        .or(admin_login_page)
        .or(admin_page)
        .or(admin_new)
        .or(admin_create)
        .or(admin_edit)
        .or(admin_update)
        .or(admin_delete)
}
