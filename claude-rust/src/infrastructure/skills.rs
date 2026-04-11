use std::path::Path;

pub struct Skill {
    pub name: String,
    pub description: String,
    pub argument_hint: Option<String>,
    pub when_to_use: Option<String>,
    pub allowed_tools: Vec<String>,
    pub user_invocable: bool,
    pub prompt_template: String,
}

impl Skill {
    pub fn expand(&self, args: &str) -> String {
        let result = if args.is_empty() {
            self.prompt_template.replace("$ARGUMENTS", "").trim().to_string()
        } else {
            self.prompt_template.replace("$ARGUMENTS", args)
        };
        // Also replace ${ARGUMENTS} variant
        result.replace("${ARGUMENTS}", if args.is_empty() { "" } else { args })
    }
}

/// Load skills from both /skills/ and legacy /commands/ directories.
///
/// Search order:
///   1. ~/.claude/skills/     (global skills — directory format: name/SKILL.md)
///   2. .claude/skills/       (project skills — directory format: name/SKILL.md)
///   3. ~/.claude/commands/   (global legacy commands — .md files + dir format)
///   4. .claude/commands/     (project legacy commands — .md files + dir format)
///
/// Works on Linux and macOS (uses HOME env var, falls back to dirs crate).
pub fn load_skills(cwd: &str) -> Vec<Skill> {
    let home = home_dir();
    let mut skills = Vec::new();

    // 1. Skills directories (directory format only: name/SKILL.md)
    let skills_dirs = [
        format!("{home}/.claude/skills"),
        format!("{cwd}/.claude/skills"),
    ];
    for dir in &skills_dirs {
        load_skills_from_dir(dir, &mut skills, true);
    }

    // 2. Legacy commands directories (.md files + directory format)
    let commands_dirs = [
        format!("{home}/.claude/commands"),
        format!("{cwd}/.claude/commands"),
    ];
    for dir in &commands_dirs {
        load_skills_from_dir(dir, &mut skills, false);
    }

    skills
}

fn home_dir() -> String {
    claude_rust_config::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}

/// Load skills from a directory.
///
/// If `skills_dir_only` is true, only loads directory-format skills (name/SKILL.md).
/// If false, also loads standalone .md files (legacy /commands/ behavior).
fn load_skills_from_dir(dir: &str, skills: &mut Vec<Skill>, skills_dir_only: bool) {
    let path = Path::new(dir);
    if !path.is_dir() {
        return;
    }
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();

        // Directory format: skill-name/SKILL.md (or skill-name/skill.md)
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
                let dir_name = p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                if let Some(skill) = parse_skill_file(&file_path, Some(&dir_name))
                    && !skills.iter().any(|s| s.name == skill.name) {
                        skills.push(skill);
                    }
            }
            continue;
        }

        // Standalone .md file (legacy commands format only)
        if !skills_dir_only
            && p.extension().and_then(|e| e.to_str()) == Some("md")
            && let Some(skill) = parse_skill_file(&p, None)
                && !skills.iter().any(|s| s.name == skill.name) {
                    skills.push(skill);
                }
    }
}

fn parse_skill_file(path: &Path, dir_name: Option<&str>) -> Option<Skill> {
    let content = std::fs::read_to_string(path).ok()?;
    let (frontmatter, body) = split_frontmatter(&content)?;

    // Name resolution: frontmatter "name" > directory name > file stem
    let name = extract_field(&frontmatter, "name")
        .or_else(|| dir_name.map(|s| s.to_string()))
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })?;

    let description = extract_field(&frontmatter, "description").unwrap_or_default();

    // argument-hint (supports both underscore and hyphen variants)
    let argument_hint = extract_field(&frontmatter, "argument-hint")
        .or_else(|| extract_field(&frontmatter, "argument_hint"));

    let when_to_use = extract_field(&frontmatter, "when_to_use")
        .or_else(|| extract_field(&frontmatter, "when-to-use"));

    // allowed-tools: comma-separated or multi-value
    let allowed_tools = extract_field(&frontmatter, "allowed-tools")
        .or_else(|| extract_field(&frontmatter, "allowed_tools"))
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // user-invocable defaults to true
    let user_invocable = extract_field(&frontmatter, "user-invocable")
        .or_else(|| extract_field(&frontmatter, "user_invocable"))
        .map(|v| !matches!(v.to_lowercase().as_str(), "false" | "no" | "0"))
        .unwrap_or(true);

    Some(Skill {
        name,
        description,
        argument_hint,
        when_to_use,
        allowed_tools,
        user_invocable,
        prompt_template: body.trim().to_string(),
    })
}

fn split_frontmatter(content: &str) -> Option<(String, String)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Some((String::new(), content.to_string()));
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let frontmatter = rest[..end].to_string();
    let body = rest[end + 4..].to_string();
    Some((frontmatter, body))
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
