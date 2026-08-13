use std::path::PathBuf;

/// 平台默认配置目录：
/// - unix: $XDG_CONFIG_HOME/agc 或 ~/.config/agc
/// - windows: %APPDATA%\agc
pub fn config_dir_default() -> Option<PathBuf> {
    let home = config_home()?;
    Some(home.join("agc"))
}

/// 平台默认数据目录：
/// - unix: $XDG_DATA_HOME/agc 或 ~/.local/share/agc
/// - windows: %APPDATA%\agc
pub fn data_dir_default() -> Option<PathBuf> {
    let home = data_home()?;
    Some(home.join("agc"))
}

#[cfg(target_family = "unix")]
fn config_home() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|v| !v.is_empty())
                .map(|h| PathBuf::from(h).join(".config"))
        })
}

#[cfg(not(target_family = "unix"))]
fn config_home() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|v| !v.is_empty())
                .map(|h| PathBuf::from(h).join(".config"))
        })
        .or_else(|| {
            std::env::var("APPDATA")
                .ok()
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        })
}

#[cfg(target_family = "unix")]
fn data_home() -> Option<PathBuf> {
    std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|v| !v.is_empty())
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        })
}

#[cfg(not(target_family = "unix"))]
fn data_home() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_env::{self, restore_var};

    #[test]
    fn config_dir_ends_with_agc() {
        let dir = config_dir_default().expect("config dir should resolve");
        assert_eq!(dir.file_name().unwrap(), "agc");
    }

    #[test]
    fn data_dir_ends_with_agc() {
        let dir = data_dir_default().expect("data dir should resolve");
        assert_eq!(dir.file_name().unwrap(), "agc");
    }

    #[cfg(not(target_family = "unix"))]
    #[test]
    fn config_dir_uses_userprofile_config_on_windows() {
        let _guard = test_env::LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let orig_user = std::env::var("USERPROFILE").ok();
        let orig_app = std::env::var("APPDATA").ok();

        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("USERPROFILE", temp.path());
        std::env::remove_var("APPDATA");

        let dir = config_dir_default().expect("config dir should resolve");
        assert_eq!(
            dir,
            temp.path().join(".config").join("agc")
        );

        restore_var("XDG_CONFIG_HOME", orig_xdg);
        restore_var("USERPROFILE", orig_user);
        restore_var("APPDATA", orig_app);
    }

    #[cfg(not(target_family = "unix"))]
    #[test]
    fn config_dir_falls_back_to_appdata() {
        let _guard = test_env::LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let orig_user = std::env::var("USERPROFILE").ok();
        let orig_app = std::env::var("APPDATA").ok();

        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("USERPROFILE");
        std::env::set_var("APPDATA", temp.path());

        let dir = config_dir_default().expect("config dir should resolve");
        assert_eq!(dir, temp.path().join("agc"));

        restore_var("XDG_CONFIG_HOME", orig_xdg);
        restore_var("USERPROFILE", orig_user);
        restore_var("APPDATA", orig_app);
    }
}
