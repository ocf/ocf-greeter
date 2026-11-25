use freedesktop_desktop_entry::{DesktopEntry, get_languages_from_env};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DesktopSession {
    pub id: String,
    pub name: String,
    pub exec: String,
}

impl fmt::Display for DesktopSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// FIXME: should walk folder tree for all .desktop files
pub fn get_desktop_sessions(dir: &Path) -> Vec<DesktopSession> {
    let locales = get_languages_from_env();

    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("desktop")) {
                items.push(get_desktop_entry(&path, &locales))
            }
        }
    }
    items
}

fn get_desktop_entry(path: &Path, locales: &[String]) -> DesktopSession {
    let entry = DesktopEntry::from_path(path, Some(locales)).unwrap();
    let id = path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("Unknown")
        .to_string();
    let name = (entry.name(locales))
        .unwrap_or(Cow::Borrowed("Unknown"))
        .to_string();
    let exec = (entry.exec()).unwrap_or("id").to_string();

    DesktopSession { id, name, exec }
}
