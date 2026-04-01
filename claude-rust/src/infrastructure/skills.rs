use std::path::Path;

pub struct Skill {
    pub name: String,
    pub description: String,
    pub argument_hint: Option<String>,
    pub prompt_template: String,
}

impl Skill {
    pub fn expand(&self, args: &str) -> String {
        if args.is_empty() {
            self.prompt_template.replace("$ARGUMENTS", "").trim().to_string()
        } else {
            self.prompt_template.replace("$ARGUMENTS", args)
        }
    }
}

pub fn load_skills(cwd: &str) -> Vec<Skill> {
    let home = std::env::var("HOME").unwrap_or_default();
    let search_dirs = [
        format!("{home}/.claude/skills"),
        format!("{cwd}/.claude/skills"),
    ];
    let mut skills = Vec::new();
    for dir in &search_dirs {
        let path = Path::new(dir);
        if !path.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Some(skill) = parse_skill_file(&p) {
                        if !skills.iter().any(|s: &Skill| s.name == skill.name) {
                            skills.push(skill);
                        }
                    }
                }
            }
        }
    }
    skills
}

fn parse_skill_file(path: &Path) -> Option<Skill> {
    let content = std::fs::read_to_string(path).ok()?;
    let (frontmatter, body) = split_frontmatter(&content)?;
    let name = extract_field(&frontmatter, "name")
        .or_else(|| path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()))?;
    let description = extract_field(&frontmatter, "description").unwrap_or_default();
    let argument_hint = extract_field(&frontmatter, "argument_hint");
    Some(Skill {
        name,
        description,
        argument_hint,
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
