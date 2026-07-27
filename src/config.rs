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
    #[error(
        "could not determine a default notes directory; set --notes-dir, ZTK_NOTES_DIR, or notes_dir in the config file"
    )]
    DefaultDirectoryUnavailable,
    #[error("failed to create notes directory `{path}`: {source}")]
    CreateNotesDirectory {
        path: PathBuf,
        source: std::io::Error,
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
    let environment = env::var_os("ZTK_NOTES_DIR").map(PathBuf::from);
    let config_path = configured_file_path();
    let default = platform_default_notes_dir(
        env::var_os("XDG_DATA_HOME"),
        env::var_os("LOCALAPPDATA"),
        env::var_os("HOME"),
    );
    resolve(explicit, environment, config_path.as_deref(), default, &cwd)
}

pub fn ensure_notes_dir(path: &Path) -> Result<(), ConfigError> {
    fs::create_dir_all(path).map_err(|source| ConfigError::CreateNotesDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn configured_file_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("ZTK_CONFIG") {
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
        .map(|path| path.join("ztk/config.toml"))
        .or_else(|| {
            app_data
                .map(PathBuf::from)
                .map(|path| path.join("ztk/config.toml"))
        })
        .or_else(|| {
            home.map(PathBuf::from)
                .map(|path| path.join(".config/ztk/config.toml"))
        })
}

fn platform_default_notes_dir(
    xdg_data: Option<OsString>,
    local_app_data: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    xdg_data
        .map(PathBuf::from)
        .map(|path| path.join("ztk/notes"))
        .or_else(|| {
            local_app_data
                .map(PathBuf::from)
                .map(|path| path.join("ztk/notes"))
        })
        .or_else(|| {
            home.map(PathBuf::from)
                .map(|path| path.join(".local/share/ztk/notes"))
        })
}

fn resolve(
    explicit: Option<PathBuf>,
    environment: Option<PathBuf>,
    config_path: Option<&Path>,
    default: Option<PathBuf>,
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

    default.ok_or(ConfigError::DefaultDirectoryUnavailable)
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
    use super::{platform_config_file, platform_default_notes_dir, resolve};
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
                Some(root.path().join("default")),
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
                Some(root.path().join("default")),
                root.path()
            )
            .unwrap(),
            root.path().join("environment")
        );
        assert_eq!(
            resolve(
                None,
                None,
                Some(&config_path),
                Some(root.path().join("default")),
                root.path(),
            )
            .unwrap(),
            config_path.parent().unwrap().join("from-config")
        );
        assert_eq!(
            resolve(
                None,
                None,
                None,
                Some(root.path().join("default")),
                root.path(),
            )
            .unwrap(),
            root.path().join("default")
        );
    }

    #[test]
    fn absolute_and_space_containing_paths_are_preserved() {
        let path = PathBuf::from("/tmp/ztk notes");
        assert_eq!(
            resolve(Some(path.clone()), None, None, None, Path::new("/work")).unwrap(),
            path
        );
    }

    #[test]
    fn malformed_and_unknown_config_fields_are_rejected() {
        let root = TempDir::new().unwrap();
        for content in ["not toml", "notes_dir = 'notes'\nunknown = true"] {
            let path = root.path().join("config.toml");
            fs::write(&path, content).unwrap();
            assert!(resolve(None, None, Some(&path), None, root.path()).is_err());
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
            Some(PathBuf::from("/xdg/ztk/config.toml"))
        );
    }

    #[test]
    fn platform_data_directory_precedence_is_deterministic() {
        assert_eq!(
            platform_default_notes_dir(
                Some(OsString::from("/xdg-data")),
                Some(OsString::from("/local-app-data")),
                Some(OsString::from("/home/user")),
            ),
            Some(PathBuf::from("/xdg-data/ztk/notes"))
        );
        assert_eq!(
            platform_default_notes_dir(None, None, Some(OsString::from("/home/user"))),
            Some(PathBuf::from("/home/user/.local/share/ztk/notes"))
        );
    }
}
