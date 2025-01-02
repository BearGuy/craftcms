use crate::models::{Image, ImageInput};
use rusqlite::Connection;
use serde_json;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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

pub fn insert_image_command(
    conn: &Connection,
    json_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the JSON input
    let input: ImageInput = serde_json::from_str(json_str)?;

    // Read the image file
    let image_path = Path::new(&input.url);
    let image_data = fs::read(image_path)?;

    // Create the Image struct
    let image = Image {
        alt: input.alt,
        description: input.description,
        slug: input.slug,
        keywords: input.keywords,
        image_data,
    };

    // Insert the image into the database
    match crate::database::insert_image(conn, &image) {
        Ok(_) => println!("Image inserted successfully!"),
        Err(e) => println!("Error inserting image: {}", e),
    }

    Ok(())
}
