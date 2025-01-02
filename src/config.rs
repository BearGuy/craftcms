use serde::Deserialize;
use std::fs;
use std::net::Ipv4Addr;
use toml;

#[derive(Debug, Deserialize, Clone)]
pub struct SiteConfig {
    pub name: String,
    pub title: String,
    pub description: String,
    pub base_url: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        SiteConfig {
            name: "My CraftCMS Site".to_string(),
            title: "Welcome to CraftCMS".to_string(),
            description:
                "A simple CMS for managing and serving templated HTML documents with images."
                    .to_string(),
            base_url: "http://localhost:8080".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoutesConfig {
    pub detail_path: String,
    pub images_path: String,
}

impl Default for RoutesConfig {
    fn default() -> Self {
        RoutesConfig {
            detail_path: "post".to_string(),
            images_path: "images".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetaConfig {
    pub author: String,
    pub creator_suffix: String,
}

impl Default for MetaConfig {
    fn default() -> Self {
        MetaConfig {
            author: "CraftCMS User".to_string(),
            creator_suffix: "Created with CraftCMS".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn get_ip_addr(&self) -> Ipv4Addr {
        self.host.parse().unwrap_or(Ipv4Addr::LOCALHOST)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub site: SiteConfig,
    pub routes: RoutesConfig,
    pub server: ServerConfig,
    pub meta: MetaConfig,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn get_detail_url(&self, slug: &str) -> String {
        format!(
            "{}/{}/{}",
            self.site.base_url, self.routes.detail_path, slug
        )
    }

    pub fn get_image_url(&self, slug: &str) -> String {
        format!(
            "{}/{}/{}",
            self.site.base_url, self.routes.images_path, slug
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            site: SiteConfig::default(),
            routes: RoutesConfig::default(),
            server: ServerConfig::default(),
            meta: MetaConfig::default(),
        }
    }
}
