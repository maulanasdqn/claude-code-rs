use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::dto::{
    ContextBundle, IngestRequest, IngestResponse, LessonList, SearchResponse, SessionList,
    SessionMeta,
};

/// Minimal async client for the Truncus Worker HTTP API. Every call is bounded
/// by a short timeout so a slow/unreachable memory service never blocks a Stynx
/// session start or turn.
pub struct ApiClient {
    base: String,
    token: String,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(config: &Config) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_default();
        Self {
            base: config.url.trim_end_matches('/').to_string(),
            token: config.token.clone(),
            http,
        }
    }

    pub async fn ingest(&self, request: &IngestRequest) -> Result<IngestResponse, String> {
        self.exec(self.http.post(self.url("/v1/sessions")).json(request))
            .await
    }

    pub async fn context(&self, project: &str) -> Result<ContextBundle, String> {
        self.exec(
            self.http
                .get(self.url("/v1/context"))
                .query(&[("project", project)]),
        )
        .await
    }

    pub async fn search(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<SearchResponse, String> {
        self.exec(self.http.get(self.url("/v1/search")).query(&self.list_params(query, project, limit)))
            .await
    }

    pub async fn knowledge(
        &self,
        query: &str,
        project: Option<&str>,
        limit: usize,
    ) -> Result<SearchResponse, String> {
        self.exec(self.http.get(self.url("/v1/knowledge")).query(&self.list_params(query, project, limit)))
            .await
    }

    pub async fn lessons(
        &self,
        project: Option<&str>,
        limit: usize,
    ) -> Result<LessonList, String> {
        let mut params = vec![("limit", limit.to_string())];
        if let Some(p) = project {
            params.push(("project", p.to_string()));
        }
        self.exec(self.http.get(self.url("/v1/lessons")).query(&params))
            .await
    }

    pub async fn sessions(
        &self,
        project: Option<&str>,
        limit: usize,
    ) -> Result<SessionList, String> {
        let mut params = vec![("limit", limit.to_string()), ("offset", "0".to_string())];
        if let Some(p) = project {
            params.push(("project", p.to_string()));
        }
        self.exec(self.http.get(self.url("/v1/sessions")).query(&params))
            .await
    }

    pub async fn session(&self, id: &str) -> Result<SessionMeta, String> {
        self.exec(self.http.get(self.url(&format!("/v1/sessions/{id}"))))
            .await
    }

    fn list_params(&self, query: &str, project: Option<&str>, limit: usize) -> Vec<(&'static str, String)> {
        let mut params = vec![("q", query.to_string()), ("limit", limit.to_string())];
        if let Some(p) = project {
            params.push(("project", p.to_string()));
        }
        params
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    async fn exec<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, String> {
        let response = request
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("api {}: {}", status.as_u16(), body.trim()));
        }
        response
            .json::<T>()
            .await
            .map_err(|e| format!("decode failed: {e}"))
    }
}
