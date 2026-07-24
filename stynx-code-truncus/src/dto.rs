//! Data transfer objects mirroring the Truncus Worker API. Some fields are
//! deserialized to stay faithful to the contract without being rendered yet.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// One flattened conversation message, as the Truncus ingest API expects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Msg {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestRequest {
    pub session_id: String,
    pub project: String,
    pub cwd: String,
    pub machine: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub messages: Vec<Msg>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestResponse {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionBrief {
    pub id: String,
    pub project: String,
    pub ended_at: i64,
    pub summary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextBundle {
    pub project_sessions: Vec<SessionBrief>,
    pub other_sessions: Vec<SessionBrief>,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
    #[serde(default)]
    pub note_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchHit {
    pub session_id: String,
    pub kind: String,
    pub score: f64,
    pub text: String,
    pub project: String,
    pub ended_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub project: String,
    pub machine: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub status: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub chunk_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionList {
    pub sessions: Vec<SessionMeta>,
    #[serde(default)]
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub project: String,
    pub category: String,
    pub title: String,
    pub insight: String,
    #[serde(default)]
    pub evidence: String,
    pub confidence: f64,
    pub times_seen: i64,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LessonList {
    pub lessons: Vec<Lesson>,
}
