use serde::{Deserialize, Serialize};

pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub alt: String,
    pub description: String,
    pub slug: String,
    pub keywords: Vec<String>,
    pub filename: String,
}

#[derive(Deserialize)]
pub struct ImageForm {
    pub alt: String,
    pub description: String,
    pub slug: String,
    pub keywords: String,
    pub image_data: Vec<u8>,
}

#[derive(Deserialize)]
pub struct ImageInput {
    #[serde(alias = "url", alias = "Url")]
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(alias = "alt", alias = "Alt")]
    pub alt: String,
    #[serde(alias = "description", alias = "Description")]
    pub description: String,
    #[serde(alias = "slug", alias = "Slug")]
    pub slug: String,
    #[serde(alias = "keywords", alias = "Keywords")]
    pub keywords: Vec<String>,
    #[serde(alias = "type", alias = "Type")]
    pub image_type: String,
}

use std::fmt;
use warp::reject::Reject;

#[derive(Debug)]
pub struct CustomError {
    pub message: String,
}

impl CustomError {
    pub fn new(message: String) -> CustomError {
        CustomError { message }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Reject for CustomError {}

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
}

#[derive(Debug)]
pub struct RedirectToLogin;
impl Reject for RedirectToLogin {}
