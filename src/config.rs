use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Error;
use crate::model::Project;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_stale_threshold")]
    stale_threshold_days: u32,
    #[serde(default)]
    projects: Vec<Project>,
}

fn default_stale_threshold() -> u32 {
    7
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    PathBuf::from(home).join(".config/reposcope/config.toml")
}

fn load_config() -> Result<Config, Error> {
    let path = config_path();

    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::ConfigIo(e.to_string()))?;
        }
        let default = Config {
            stale_threshold_days: 7,
            projects: Vec::new(),
        };
        let contents = toml::to_string(&default).map_err(|e| Error::ConfigParse(e.to_string()))?;
        std::fs::write(&path, &contents).map_err(|e| Error::ConfigIo(e.to_string()))?;
        return Ok(default);
    }

    let contents = std::fs::read_to_string(&path).map_err(|e| Error::ConfigIo(e.to_string()))?;
    toml::from_str(&contents).map_err(|e| Error::ConfigParse(e.to_string()))
}

fn save_config(config: &Config) -> Result<(), Error> {
    let path = config_path();
    let contents = toml::to_string(config).map_err(|e| Error::ConfigParse(e.to_string()))?;
    std::fs::write(&path, &contents).map_err(|e| Error::ConfigIo(e.to_string()))
}

fn expand_tilde(path: &std::path::Path) -> PathBuf {
    if let Ok(strip) = path.strip_prefix("~") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(strip);
        }
    }
    path.to_path_buf()
}

pub fn add_project(path_str: &str) -> Result<(), Error> {
    let path = expand_tilde(std::path::Path::new(path_str));
    let canonical =
        std::fs::canonicalize(&path).map_err(|_| Error::NotARepo(path_str.to_string()))?;

    git2::Repository::open(&canonical).map_err(|_| Error::NotARepo(path_str.to_string()))?;

    let mut config = load_config()?;

    for p in &config.projects {
        if p.path == canonical {
            return Err(Error::DuplicatePath(path_str.to_string()));
        }
        if let Ok(existing) = std::fs::canonicalize(&p.path) {
            if existing == canonical {
                return Err(Error::DuplicatePath(path_str.to_string()));
            }
        }
    }

    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_str.to_string());

    config.projects.push(Project {
        name,
        path: canonical,
        stale_threshold_days: None,
    });

    save_config(&config)
}

pub fn remove_project(name_or_path: &str) -> Result<(), Error> {
    let mut config = load_config()?;
    let remove_path = expand_tilde(std::path::Path::new(name_or_path));
    let len_before = config.projects.len();

    config.projects.retain(|p| {
        if p.name == name_or_path
            || p.path == remove_path
            || p.path.to_string_lossy() == name_or_path
        {
            return false;
        }
        if let Ok(canonical) = std::fs::canonicalize(&remove_path) {
            if let Ok(p_canonical) = std::fs::canonicalize(&p.path) {
                if p_canonical == canonical {
                    return false;
                }
            }
        }
        true
    });

    if config.projects.len() == len_before {
        return Err(Error::NotFound(name_or_path.to_string()));
    }

    save_config(&config)
}

pub fn list_projects() -> Result<Vec<Project>, Error> {
    let config = load_config()?;
    Ok(projects_with_effective_threshold(config))
}

fn projects_with_effective_threshold(config: Config) -> Vec<Project> {
    let global_threshold = config.stale_threshold_days;

    config
        .projects
        .into_iter()
        .map(|mut project| {
            if project.stale_threshold_days.is_none() {
                project.stale_threshold_days = Some(global_threshold);
            }
            project
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_global_stale_threshold_to_projects_without_override() {
        let projects = projects_with_effective_threshold(Config {
            stale_threshold_days: 14,
            projects: vec![project("default", None), project("custom", Some(3))],
        });

        assert_eq!(projects[0].stale_threshold_days, Some(14));
        assert_eq!(projects[1].stale_threshold_days, Some(3));
    }

    fn project(name: &str, stale_threshold_days: Option<u32>) -> Project {
        Project {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{name}")),
            stale_threshold_days,
        }
    }
}
