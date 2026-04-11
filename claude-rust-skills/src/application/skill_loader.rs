use std::path::{Path, PathBuf};

use claude_rust_errors::{AppError, AppResult};

use crate::domain::bundled_skill::bundled_skills;
use crate::domain::skill::{Skill, SkillMetadata, SkillSource};


pub struct SkillLoader;

impl SkillLoader {
    pub fn new() -> Self {
        Self
    }

    /// Load all skills from a single directory.
    pub fn load_from_directory(&self, dir: &Path) -> AppResult<Vec<Skill>> {
        let mut skills = Vec::new();
        if !dir.is_dir() {
            return Ok(skills);
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read dir {:?}: {}", dir, e)))?;

        for entry in entries.flatten() {
            let p = entry.path();

            // Directory format: skill-name/SKILL.md
            if p.is_dir() {
                let skill_file = p.join("SKILL.md");
                let skill_file_lower = p.join("skill.md");
                let actual = if skill_file.is_file() {
                    Some(skill_file)
                } else if skill_file_lower.is_file() {
                    Some(skill_file_lower)
                } else {
                    None
                };
                if let Some(file_path) = actual {
                    if let Ok(skill) = parse_skill_file(&file_path) {
                        if !skills.iter().any(|s: &Skill| s.metadata.name == skill.metadata.name) {
                            skills.push(skill);
                        }
                    }
                }
                continue;
            }

            // Standalone .md file
            if p.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(skill) = parse_skill_file(&p) {
                    if !skills.iter().any(|s: &Skill| s.metadata.name == skill.metadata.name) {
                        skills.push(skill);
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Load all skills: user skills, project skills, legacy commands, and bundled.
    pub fn load_all(&self) -> AppResult<Vec<Skill>> {
        let home = claude_rust_config::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        let mut all: Vec<Skill> = Vec::new();

        let dirs: Vec<(PathBuf, bool)> = vec![
            (PathBuf::from(format!("{home}/.claude/skills")), true),
            (PathBuf::from(".claude/skills"), false),
            (PathBuf::from(format!("{home}/.claude/commands")), true),
            (PathBuf::from(".claude/commands"), false),
        ];

        for (dir, is_user) in dirs {
            if !dir.is_dir() {
                continue;
            }
            let skills = self.load_from_directory(&dir)?;
            for mut skill in skills {
                if !all.iter().any(|s| s.metadata.name == skill.metadata.name) {
                    skill.source = if is_user {
                        SkillSource::UserSkill(dir.clone())
                    } else {
                        SkillSource::ProjectSkill(dir.clone())
                    };
                    all.push(skill);
                }
            }
        }

        // Append bundled skills (lowest priority — only if not already defined)
        for skill in bundled_skills() {
            if !all.iter().any(|s| s.metadata.name == skill.metadata.name) {
                all.push(skill);
            }
        }

        Ok(all)
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a skill from a markdown file with optional YAML frontmatter.
pub fn parse_skill_file(path: &Path) -> AppResult<Skill> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read {:?}: {}", path, e)))?;

    let (frontmatter, body) = split_frontmatter(&raw);

    // Determine name: frontmatter > file stem
    let name = extract_field(&frontmatter, "name").or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }).ok_or_else(|| AppError::BadRequest(format!("Cannot determine name for {:?}", path)))?;

    let description = extract_field(&frontmatter, "description").unwrap_or_default();

    let triggers = extract_field(&frontmatter, "triggers")
        .map(|v| {
            v.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let model = extract_field(&frontmatter, "model");

    let is_hidden = extract_field(&frontmatter, "is_hidden")
        .or_else(|| extract_field(&frontmatter, "hidden"))
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "yes" | "1"))
        .unwrap_or(false);

    Ok(Skill {
        metadata: SkillMetadata {
            name,
            description,
            triggers,
            model,
            is_hidden,
        },
        content: body.trim().to_string(),
        // Caller may override source; default to user path
        source: SkillSource::UserSkill(path.to_path_buf()),
    })
}

fn split_frontmatter(content: &str) -> (String, String) {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return (String::new(), content.to_string());
    }
    let rest = &content[3..];
    if let Some(end) = rest.find("\n---") {
        let frontmatter = rest[..end].to_string();
        let body = rest[end + 4..].to_string();
        (frontmatter, body)
    } else {
        (String::new(), content.to_string())
    }
}

fn extract_field(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
            let val = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}
