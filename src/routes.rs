use crate::config::Config;
use crate::files::ImageFileManager;
use crate::{
    handlers::*,
    middleware::{with_auth, with_file_manager},
    with_config, with_db,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use warp::Filter;

use crate::admin_assets::AdminAssets;

pub fn admin_routes(
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
    file_manager: Arc<ImageFileManager>,
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
        .and(warp::query::<crate::models::AdminImageQuery>())
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
        .and(with_file_manager(file_manager.clone())) // Add this line
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
        .and(with_file_manager(file_manager.clone()))
        .and_then(admin_update_image_handler);

    // Delete image endpoint
    let admin_delete = admin_base
        .and(warp::path("delete"))
        .and(warp::path::param())
        .and(with_auth(conn.clone()))
        .and(with_db(conn.clone()))
        .and(with_file_manager(file_manager.clone()))
        .and_then(admin_delete_image_handler);

    let admin_restore = admin_base
        .and(warp::path("restore"))
        .and(warp::path::param())
        .and(with_auth(conn.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_restore_image_handler);

    let admin_status = admin_base
        .and(warp::path("status"))
        .and(warp::path::param())
        .and(with_auth(conn.clone()))
        .and(warp::body::form())
        .and(with_db(conn.clone()))
        .and_then(admin_update_status_handler);

    let admin_settings = admin_base
        .and(warp::path("settings"))
        .and(warp::path::end())
        .and(with_auth(conn.clone()))
        .and(with_config(config.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_settings_page_handler);

    let admin_settings_update = admin_base
        .and(warp::path("settings"))
        .and(warp::path("update"))
        .and(with_auth(conn.clone()))
        .and(warp::body::form())
        .and(with_db(conn.clone()))
        .and_then(admin_update_settings_handler);

    let instagram_disconnect = admin_base
        .and(warp::path("settings"))
        .and(warp::path("instagram"))
        .and(warp::path("disconnect"))
        .and(with_auth(conn.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_instagram_disconnect_handler);

    let instagram_sync = admin_base
        .and(warp::path("settings"))
        .and(warp::path("instagram"))
        .and(warp::path("sync"))
        .and(with_auth(conn.clone()))
        .and(with_config(config.clone()))
        .and(with_db(conn.clone()))
        .and(with_file_manager(file_manager.clone()))
        .and_then(admin_instagram_sync_handler);

    let instagram_refresh = admin_base
        .and(warp::path("settings"))
        .and(warp::path("instagram"))
        .and(warp::path("refresh"))
        .and(with_auth(conn.clone()))
        .and(with_db(conn.clone()))
        .and_then(admin_instagram_refresh_handler);

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
        .or(admin_restore)
        .or(admin_status)
        .or(admin_settings)
        .or(admin_settings_update)
        .or(instagram_disconnect)
        .or(instagram_sync)
        .or(instagram_refresh)
}
