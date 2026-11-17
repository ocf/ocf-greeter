use freedesktop_desktop_entry::DesktopEntry;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DesktopSession {
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
    let mut items = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("desktop")) {
                items.push(get_desktop_entry(&path))
            }
        }
    }
    items
}

// FIXME: set locale dynamically
fn get_desktop_entry(path: &Path) -> DesktopSession {
    let entry = DesktopEntry::from_path(path, Some(&["en_US.UTF-8"])).unwrap();
    let name = (entry.name(&["en_US.UTF-8"]))
        .unwrap_or(Cow::Borrowed("Unknown"))
        .to_string();
    let exec = (entry.exec()).unwrap_or("id").to_string();

    DesktopSession { name, exec }
}
