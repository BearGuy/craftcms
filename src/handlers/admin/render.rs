use crate::config::Config;
use crate::models::{AdminImageQuery, CustomError};
use crate::template_utils::render_template;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tera::Context;
use warp::Reply;

pub async fn admin_login_page_handler(config: Arc<Config>) -> Result<impl Reply, warp::Rejection> {
    let mut context = Context::new();
    context.insert("site_name", &config.site.name);
    render_template("admin/login.html", &context).await
}

pub async fn admin_page_handler(
    query: AdminImageQuery,
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut context = Context::new();
    let active_filter = query.filter.unwrap_or_else(|| "all".to_string());

    let images = {
        let conn_guard = conn.lock().map_err(|e| {
            eprintln!("Failed to lock mutex: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Internal server error".to_string(),
            })
        })?;

        crate::database::get_admin_images(&conn_guard, &active_filter).map_err(|e| {
            eprintln!("Failed to get images: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Failed to load images".to_string(),
            })
        })?
    };

    // Add all config-related context
    context.insert("site_name", &config.site.name);
    context.insert("title", &format!("Admin - {}", &config.site.name));
    context.insert("base_url", &config.site.base_url);
    context.insert("detail_path", &config.routes.detail_path);
    context.insert("images_path", &config.routes.images_path);
    context.insert("images", &images);
    context.insert("active_filter", &active_filter);

    render_template("admin/admin.html", &context).await
}

pub async fn admin_new_image_handler(config: Arc<Config>) -> Result<impl Reply, warp::Rejection> {
    let mut context = Context::new();

    context.insert("site_name", &config.site.name);
    context.insert("title", &format!("Admin - {}", &config.site.name));
    context.insert("base_url", &config.site.base_url);
    context.insert("detail_path", &config.routes.detail_path);
    context.insert("images_path", &config.routes.images_path);

    render_template("admin/admin_new_image.html", &context).await
}

pub async fn admin_edit_image_handler(
    slug: String,
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut context = Context::new();

    context.insert("site_name", &config.site.name);
    context.insert("title", &format!("Admin - {}", &config.site.name));
    context.insert("base_url", &config.site.base_url);
    context.insert("detail_path", &config.routes.detail_path);
    context.insert("images_path", &config.routes.images_path);

    // Load the image by slug
    let image = {
        let conn_guard = conn.lock().map_err(|e| {
            eprintln!("Failed to lock mutex: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Internal server error".to_string(),
            })
        })?;

        crate::database::get_image_by_slug(&conn_guard, &slug).map_err(|e| {
            eprintln!("Failed to get image: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Image not found".to_string(),
            })
        })?
    };
    context.insert("image", &image);

    render_template("admin/admin_edit_image.html", &context).await
}

pub async fn admin_settings_page_handler(
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut context = Context::new();

    let (settings, instagram_connection) = {
        let conn_guard = conn.lock().map_err(|e| {
            eprintln!("Failed to lock mutex: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Internal server error".to_string(),
            })
        })?;

        let settings = crate::database::get_app_settings(&conn_guard).map_err(|e| {
            eprintln!("Failed to load settings: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Failed to load settings".to_string(),
            })
        })?;

        let instagram_connection =
            crate::database::get_instagram_connection(&conn_guard).map_err(|e| {
                eprintln!("Failed to load instagram connection: {:?}", e);
                warp::reject::custom(CustomError {
                    message: "Failed to load Instagram connection".to_string(),
                })
            })?;

        (settings, instagram_connection)
    };

    context.insert("site_name", &config.site.name);
    context.insert("title", &format!("Settings - {}", &config.site.name));
    context.insert("settings", &settings);
    context.insert("instagram_connection", &instagram_connection);
    render_template("admin/admin_settings.html", &context).await
}
