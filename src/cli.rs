use crate::files::ImageFileManager;
use crate::models::{Image, ImageInput};
use rusqlite::Connection;
use serde_json;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub fn create_user_command(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    print!("Enter password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;

    print!("Confirm password: ");
    io::stdout().flush()?;
    let confirm_password = rpassword::read_password()?;

    if password != confirm_password {
        println!("Passwords do not match!");
        return Ok(());
    }

    match crate::database::create_user(conn, email, &password) {
        Ok(_) => println!("User created successfully!"),
        Err(e) => println!("Error creating user: {}", e),
    }

    Ok(())
}

pub fn list_users_command(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let users = crate::database::list_users(conn)?;
    println!("\nRegistered Users:");
    println!("----------------");
    for user in users {
        println!("Email: {}", user.email);
    }
    Ok(())
}

pub fn delete_user_command(
    conn: &Connection,
    email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("Are you sure you want to delete user {}? (y/N): ", email);
    io::stdout().flush()?;

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() == "y" {
        crate::database::delete_user(conn, email)?;
        println!("User deleted successfully!");
    } else {
        println!("Operation cancelled.");
    }

    Ok(())
}

pub fn handle_insert_image(
    conn: &Connection,
    file_manager: &ImageFileManager,
    json_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let input: ImageInput = serde_json::from_str(json_str)?;
    let image = Image {
        alt: input.alt,
        description: input.description,
        slug: input.slug,
        keywords: input.keywords,
        filename: String::new(), // Will be set during save
        status: "published".to_string(),
        deleted_at: None,
        source: "manual".to_string(),
        source_media_id: None,
        source_permalink: None,
        source_timestamp: None,
    };

    // Use insert_image_from_path since we have a file path in input.url
    crate::commands::insert_image_from_path(conn, file_manager, &input.url, image)
}

pub fn regenerate_image_variants_command(
    conn: &Connection,
    file_manager: &ImageFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let images = crate::database::get_admin_images(conn, "all")?;
    let mut regenerated = 0usize;
    for image in images {
        file_manager.regenerate_variants(&image.filename)?;
        regenerated += 1;
    }
    println!("Regenerated variants for {} images.", regenerated);
    Ok(())
}

pub fn regenerate_missing_image_variants_command(
    conn: &Connection,
    file_manager: &ImageFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let images = crate::database::get_admin_images(conn, "all")?;
    let total = images.len();
    let mut regenerated = 0usize;
    for image in images {
        if file_manager.regenerate_missing_variants(&image.filename)? {
            regenerated += 1;
        }
    }
    println!(
        "Regenerated missing variants for {} of {} images.",
        regenerated, total
    );
    Ok(())
}

pub async fn sync_instagram_command(
    conn: Connection,
    file_manager: ImageFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = crate::services::instagram_importer::sync_instagram(
        Arc::new(Mutex::new(conn)),
        Arc::new(file_manager),
    )
    .await?;
    println!("{}", result.message());
    Ok(())
}

pub async fn refresh_instagram_token_command(
    conn: Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let result =
        crate::services::instagram_importer::refresh_instagram_token(Arc::new(Mutex::new(conn)))
            .await?;
    println!(
        "Instagram token refreshed for @{} until {}.",
        result.username, result.token_expires_at
    );
    Ok(())
}
