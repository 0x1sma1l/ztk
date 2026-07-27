use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file `{path}`: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid config file `{path}`: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    notes_dir: PathBuf,
}

pub fn resolve_notes_dir(explicit: Option<PathBuf>) -> Result<PathBuf, ConfigError> {
    let cwd = env::current_dir().map_err(|source| ConfigError::Read {
        path: PathBuf::from("."),
        source,
    })?;
    let environment = env::var_os("ZET_NOTES_DIR").map(PathBuf::from);
    let config_path = configured_file_path();
    resolve(explicit, environment, config_path.as_deref(), &cwd)
}

fn configured_file_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("ZET_CONFIG") {
        return Some(PathBuf::from(path));
    }

    platform_config_file(
        env::var_os("XDG_CONFIG_HOME"),
        env::var_os("APPDATA"),
        env::var_os("HOME"),
    )
    .filter(|path| path.is_file())
}

fn platform_config_file(
    xdg: Option<OsString>,
    app_data: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    xdg.map(PathBuf::from)
        .map(|path| path.join("zet/config.toml"))
        .or_else(|| {
            app_data
                .map(PathBuf::from)
                .map(|path| path.join("zet/config.toml"))
        })
        .or_else(|| {
            home.map(PathBuf::from)
                .map(|path| path.join(".config/zet/config.toml"))
        })
}

fn resolve(
    explicit: Option<PathBuf>,
    environment: Option<PathBuf>,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<PathBuf, ConfigError> {
    if let Some(path) = explicit.or(environment) {
        return Ok(resolve_relative(path, cwd));
    }

    if let Some(config_path) = config_path {
        let raw = fs::read_to_string(config_path).map_err(|source| ConfigError::Read {
            path: config_path.to_path_buf(),
            source,
        })?;
        let config: FileConfig = toml::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: config_path.to_path_buf(),
            source,
        })?;
        let base = config_path.parent().unwrap_or(cwd);
        return Ok(resolve_relative(config.notes_dir, base));
    }

    Ok(cwd.join("notes"))
}

fn resolve_relative(path: PathBuf, base: &Path) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{platform_config_file, resolve};
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[test]
    fn precedence_is_explicit_then_environment_then_config_then_default() {
        let root = TempDir::new().unwrap();
        let config_path = root.path().join("config/config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, "notes_dir = 'from-config'").unwrap();

        assert_eq!(
            resolve(
                Some("explicit".into()),
                Some("environment".into()),
                Some(&config_path),
                root.path()
            )
            .unwrap(),
            root.path().join("explicit")
        );
        assert_eq!(
            resolve(
                None,
                Some("environment".into()),
                Some(&config_path),
                root.path()
            )
            .unwrap(),
            root.path().join("environment")
        );
        assert_eq!(
            resolve(None, None, Some(&config_path), root.path()).unwrap(),
            config_path.parent().unwrap().join("from-config")
        );
        assert_eq!(
            resolve(None, None, None, root.path()).unwrap(),
            root.path().join("notes")
        );
    }

    #[test]
    fn absolute_and_space_containing_paths_are_preserved() {
        let path = PathBuf::from("/tmp/zet notes");
        assert_eq!(
            resolve(Some(path.clone()), None, None, Path::new("/work")).unwrap(),
            path
        );
    }

    #[test]
    fn malformed_and_unknown_config_fields_are_rejected() {
        let root = TempDir::new().unwrap();
        for content in ["not toml", "notes_dir = 'notes'\nunknown = true"] {
            let path = root.path().join("config.toml");
            fs::write(&path, content).unwrap();
            assert!(resolve(None, None, Some(&path), root.path()).is_err());
        }
    }

    #[test]
    fn platform_config_precedence_is_deterministic() {
        assert_eq!(
            platform_config_file(
                Some(OsString::from("/xdg")),
                Some(OsString::from("/appdata")),
                Some(OsString::from("/home/user")),
            ),
            Some(PathBuf::from("/xdg/zet/config.toml"))
        );
    }
}
