pub(crate) mod error;

use reqwest::Client;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

pub use error::CrawlCacheError;

pub struct CrawlCache {
    user_agent: String,
    duration: Option<Duration>,
}

impl CrawlCache {
    pub fn new() -> Self {
        CrawlCache {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".to_string(),
            duration: None,
        }
    }
    pub fn set_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }
    pub fn set_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    pub async fn get_content(&self, url: &str) -> Result<String, CrawlCacheError> {
        let client = self.client()?;
        let content = match client.get(url).send().await {
            Ok(v) => v.text().await,
            Err(e) => {
                return Err(CrawlCacheError::Client(e.to_string()));
            }
        };
        match content {
            Ok(v) => Ok(v),
            Err(e) => Err(CrawlCacheError::Client(e.to_string())),
        }
    }
    pub async fn get_content_or_storage(
        &self,
        url: &str,
        cache_path: &str,
    ) -> Result<String, CrawlCacheError> {
        let path = Path::new(cache_path);
        if path.is_file() {
            return Ok(std::fs::read_to_string(cache_path).unwrap());
        }

        let dir_path = path.parent().unwrap().display().to_string();
        if !Path::new(dir_path.as_str()).is_dir() {
            fs::create_dir_all(dir_path.as_str()).unwrap();
        }

        let content = self.get_content(url).await?;
        let mut file = File::create(cache_path).unwrap();
        file.write(content.as_bytes()).unwrap();
        match file.flush() {
            Ok(_) => {}
            Err(_) => {}
        };
        if let Some(ref duration) = self.duration {
            tokio::time::sleep(duration.clone()).await;
        }
        Ok(content)
    }

    fn client(&self) -> Result<Client, CrawlCacheError> {
        let res = reqwest::ClientBuilder::new()
            .user_agent(&self.user_agent)
            .build();
        match res {
            Ok(v) => Ok(v),
            Err(e) => Err(CrawlCacheError::Client(e.to_string())),
        }
    }
}

#[tokio::test]
async fn test() {
    let cache = CrawlCache::new();
    let a = cache
        .get_content_or_storage("https://www.yahoo.co.jp/", "./yahoo.co.jp/index.html")
        .await;
    assert!(a.is_ok())
}
