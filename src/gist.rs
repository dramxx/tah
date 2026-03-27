use crate::config::Config;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

const BASE_URL: &str = "https://api.github.com";
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Deserialize)]
pub struct GistFile {
    pub content: Option<String>,
    pub truncated: Option<bool>,
    pub raw_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GistResponse {
    pub files: HashMap<String, GistFile>,
}

pub struct GistClient {
    client: Client,
    token: String,
    gist_id: String,
}

impl GistClient {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            token: config.token.clone(),
            gist_id: config.gist_id.clone(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().unwrap(),
        );
        headers.insert(reqwest::header::USER_AGENT, "tah-cli".parse().unwrap());
        headers
    }

    fn send_with_retry<F, T>(&self, request: F) -> Result<T, GistError>
    where
        F: Fn() -> Result<T, GistError>,
    {
        let mut attempts = 0;
        loop {
            match request() {
                Err(GistError::Http(500..=599)) if attempts < MAX_RETRIES => {
                    attempts += 1;
                    std::thread::sleep(Duration::from_millis(500 * attempts as u64));
                }
                result => return result,
            }
        }
    }

    pub fn get_gist(&self) -> Result<GistResponse, GistError> {
        let url = format!("{}/gists/{}", BASE_URL, self.gist_id);
        self.send_with_retry(|| {
            let response = self
                .client
                .get(&url)
                .headers(self.headers())
                .send()
                .map_err(GistError::Network)?;

            match response.status().as_u16() {
                200 => response.json().map_err(|_| GistError::Parse),
                401 => Err(GistError::InvalidToken),
                404 => Err(GistError::GistNotFound),
                status => Err(GistError::Http(status)),
            }
        })
    }

    pub fn update_file(&self, filename: &str, content: &str) -> Result<GistResponse, GistError> {
        #[derive(Serialize)]
        struct UpdateRequest {
            files: HashMap<String, FileUpdate>,
        }

        #[derive(Serialize)]
        struct FileUpdate {
            content: String,
        }

        let url = format!("{}/gists/{}", BASE_URL, self.gist_id);
        let mut files = HashMap::new();
        files.insert(
            filename.to_string(),
            FileUpdate {
                content: content.to_string(),
            },
        );

        let request = UpdateRequest { files };
        
        self.send_with_retry(|| {
            let response = self
                .client
                .patch(&url)
                .headers(self.headers())
                .json(&request)
                .send()
                .map_err(GistError::Network)?;

            match response.status().as_u16() {
                200 => response.json().map_err(|_| GistError::Parse),
                401 => Err(GistError::InvalidToken),
                404 => Err(GistError::GistNotFound),
                status => Err(GistError::Http(status)),
            }
        })
    }

    fn get_raw_url_content(&self, raw_url: &str) -> Result<String, GistError> {
        self.send_with_retry(|| {
            let response = self
                .client
                .get(raw_url)
                .headers(self.headers())
                .send()
                .map_err(GistError::Network)?;

            match response.status().as_u16() {
                200 => response.text().map_err(GistError::Network),
                401 => Err(GistError::InvalidToken),
                404 => Err(GistError::GistNotFound),
                status => Err(GistError::Http(status)),
            }
        })
    }

    pub fn file_content(&self, gist: &GistResponse, filename: &str) -> Result<String, GistError> {
        let Some(file) = gist.files.get(filename) else {
            return Ok(String::new());
        };

        if file.truncated.unwrap_or(false) {
            if let Some(raw_url) = &file.raw_url {
                return self.get_raw_url_content(raw_url);
            }
            return Err(GistError::Parse);
        }

        if let Some(content) = &file.content {
            return Ok(content.clone());
        }

        if let Some(raw_url) = &file.raw_url {
            return self.get_raw_url_content(raw_url);
        }

        Ok(String::new())
    }

    pub fn get_file_content(&self, filename: &str) -> Result<String, GistError> {
        let gist = self.get_gist()?;
        self.file_content(&gist, filename)
    }

    pub fn append_to_file(&self, filename: &str, new_content: &str) -> Result<GistResponse, GistError> {
        let mut attempts = 0;

        loop {
            let current_content = self.get_file_content(filename)?;
            let updated_content = if current_content.is_empty() {
                new_content.to_string()
            } else if current_content.ends_with('\n') {
                format!("{}{}", current_content, new_content)
            } else {
                format!("{}\n{}", current_content, new_content)
            };

            let result = self.update_file(filename, &updated_content);
            
            match result {
                Ok(response) => return Ok(response),
                Err(GistError::Http(409)) if attempts < MAX_RETRIES => {
                    attempts += 1;
                    std::thread::sleep(Duration::from_millis(200 * attempts as u64));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[derive(Debug)]
pub enum GistError {
    Network(reqwest::Error),
    Parse,
    InvalidToken,
    GistNotFound,
    Http(u16),
}

impl std::fmt::Display for GistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GistError::Network(e) => write!(f, "error: could not reach GitHub API: {}", e),
            GistError::Parse => write!(f, "error: could not parse response"),
            GistError::InvalidToken => write!(f, "error: invalid token. Re-run 'tah --init'"),
            GistError::GistNotFound => write!(f, "error: gist not found. Check your gist ID."),
            GistError::Http(code) => write!(f, "error: HTTP {}", code),
        }
    }
}

impl std::error::Error for GistError {}
