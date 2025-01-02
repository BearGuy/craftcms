use clap::{Parser, Subcommand};
use craftcms::{cli, database, run_server, setup_database};
use std::io::Read; // Add this import

#[derive(Parser)]
#[clap(
    name = "craftcms",
    version = "0.1.0",
    about = "A simple CMS for managing and serving templated HTML documents with images."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server
    Serve,
    /// Initialize the database
    InitDb,
    /// User management commands
    Users {
        #[clap(subcommand)]
        command: UserCommands,
    },
    /// Image management commands
    Images {
        #[clap(subcommand)]
        command: ImageCommands,
    },
}

#[derive(Subcommand)]
enum UserCommands {
    /// Create a new user
    Create,
    /// List all users
    List,
    // /// Delete a user
    Delete {
        #[clap(help = "Email of the user to delete")]
        email: String,
    },
}

#[derive(Subcommand)]
enum ImageCommands {
    /// Insert a new image from JSON input
    Insert,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve => {
            println!("Starting server...");
            run_server().await;
        }
        Commands::InitDb => {
            println!("Initializing database...");
            setup_database().expect("Failed to initialize database");
        }
        Commands::Users { command } => {
            let conn = database::init_db().expect("Failed to open database");
            match command {
                UserCommands::Create => {
                    if let Err(e) = cli::create_user_command(&conn) {
                        eprintln!("Error creating user: {}", e);
                        std::process::exit(1);
                    }
                }
                UserCommands::List => {
                    if let Err(e) = cli::list_users_command(&conn) {
                        eprintln!("Error listing users: {}", e);
                        std::process::exit(1);
                    }
                }
                UserCommands::Delete { email } => {
                    if let Err(e) = cli::delete_user_command(&conn, &email) {
                        eprintln!("Error deleting user: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Images { command } => {
            let conn = database::init_db().expect("Failed to open database");
            match command {
                ImageCommands::Insert => {
                    if !atty::is(atty::Stream::Stdin) {
                        let mut input = String::new();
                        std::io::stdin()
                            .read_to_string(&mut input)
                            .expect("Failed to read from stdin");

                        if let Err(e) = cli::insert_image_command(&conn, &input) {
                            eprintln!("Error inserting image: {}", e);
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!(
                            "No input detected. Usage: cat image.json | craftcms images insert"
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
