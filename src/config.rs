use serde::Deserialize;
use std::fs;
use toml;

#[derive(Debug, Deserialize, Clone)]
pub struct SiteConfig {
    pub name: String,
    pub title: String,
    pub description: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RoutesConfig {
    pub detail_path: String,
    pub images_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetaConfig {
    pub author: String,
    pub creator_suffix: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub site: SiteConfig,
    pub routes: RoutesConfig,
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
