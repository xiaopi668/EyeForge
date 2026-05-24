use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ImportedSkill {
    pub name: String,
    pub path: PathBuf,
}

pub fn import_skill(source: &str) -> Result<ImportedSkill, String> {
    let source = PathBuf::from(source.trim().trim_matches('"'));
    if source.as_os_str().is_empty() {
        return Err("请先填写 Skill 目录或 ZIP 文件路径".into());
    }

    if source.is_dir() {
        import_directory(&source)
    } else if source.is_file() {
        import_zip(&source)
    } else {
        Err(format!("路径不存在: {}", source.display()))
    }
}

fn import_directory(source: &Path) -> Result<ImportedSkill, String> {
    let skill_root = find_skill_root(source)
        .ok_or_else(|| "目录中没有找到 SKILL.md，无法识别为 Skill".to_string())?;
    let name = skill_name(&skill_root)?;
    let target = skills_dir()?.join(&name);

    if target.exists() {
        fs::remove_dir_all(&target).map_err(|error| format!("覆盖旧 Skill 失败: {error}"))?;
    }
    copy_dir_all(&skill_root, &target).map_err(|error| format!("复制 Skill 目录失败: {error}"))?;

    Ok(ImportedSkill { name, path: target })
}

fn import_zip(source: &Path) -> Result<ImportedSkill, String> {
    let file = fs::File::open(source).map_err(|error| format!("打开 ZIP 失败: {error}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| format!("读取 ZIP 失败: {error}"))?;
    let temp = skills_dir()?.join(".import-tmp");

    if temp.exists() {
        fs::remove_dir_all(&temp).map_err(|error| format!("清理临时目录失败: {error}"))?;
    }
    fs::create_dir_all(&temp).map_err(|error| format!("创建临时目录失败: {error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取 ZIP 条目失败: {error}"))?;
        let Some(relative) = safe_zip_path(entry.name()) else {
            continue;
        };
        let output = temp.join(relative);

        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|error| format!("创建目录失败: {error}"))?;
            continue;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建目录失败: {error}"))?;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| format!("读取 ZIP 文件内容失败: {error}"))?;
        fs::write(&output, bytes).map_err(|error| format!("写入 ZIP 文件内容失败: {error}"))?;
    }

    let result = import_directory(&temp);
    let _ = fs::remove_dir_all(&temp);
    result
}

fn find_skill_root(source: &Path) -> Option<PathBuf> {
    if source.join("SKILL.md").is_file() {
        return Some(source.to_path_buf());
    }

    let entries = fs::read_dir(source).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            return Some(path);
        }
    }

    None
}

fn skill_name(root: &Path) -> Result<String, String> {
    let fallback = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("imported-skill");

    let content = fs::read_to_string(root.join("SKILL.md")).unwrap_or_default();
    let mut in_frontmatter = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            if let Some(value) = trimmed.strip_prefix("name:") {
                return Ok(sanitize_name(value.trim().trim_matches('"')));
            }
        }
    }

    Ok(sanitize_name(fallback))
}

fn sanitize_name(value: &str) -> String {
    let name = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if name.is_empty() {
        "imported-skill".into()
    } else {
        name
    }
}

fn skills_dir() -> Result<PathBuf, String> {
    let path = std::env::current_dir()
        .map_err(|error| format!("获取当前目录失败: {error}"))?
        .join("skills");
    fs::create_dir_all(&path).map_err(|error| format!("创建 skills 目录失败: {error}"))?;
    Ok(path)
}

fn copy_dir_all(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn safe_zip_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return None;
    }

    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }

    if safe.as_os_str().is_empty() {
        None
    } else {
        Some(safe)
    }
}
