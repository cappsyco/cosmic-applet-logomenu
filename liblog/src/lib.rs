// SPDX-License-Identifier: GPL-3.0-only

use cosmic_config::{CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use freedesktop_desktop_entry::{
    DesktopEntry, Iter, default_paths, find_app_by_id, get_languages_from_env,
};
use once_cell::sync::Lazy;
use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io;
use std::path::Path;
use std::process::Command;

pub mod i18n;
pub use crate::i18n::init;

// Config
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, CosmicConfigEntry)]
#[version = 1]
pub struct LogoMenuConfig {
    pub logo: String,
    pub menu_items: MenuItems,
    pub custom_logo_active: bool,
    pub custom_logo_path: String,
}
impl Default for LogoMenuConfig {
    fn default() -> Self {
        Self {
            logo: String::from("Cosmic (Symbolic)"),
            menu_items: MenuItems::default(),
            custom_logo_active: false,
            custom_logo_path: String::from(""),
        }
    }
}

// Menu item types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum MenuItemType {
    LaunchAction,
    DefaultAction,
    PowerAction,
    Divider,
}
impl MenuItemType {
    pub fn as_localized_string(&self) -> String {
        match self {
            MenuItemType::LaunchAction => fl!("launch-action"),
            MenuItemType::DefaultAction => fl!("default-app"),
            MenuItemType::PowerAction => fl!("power-action"),
            MenuItemType::Divider => fl!("divider"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Hash)]
pub enum DefaultAppOption {
    WebBrowser,
    FileManager,
    MailClient,
    Music,
    Video,
    Photos,
    Calendar,
    Terminal,
    TextEditor,
}
// localised option strings
impl DefaultAppOption {
    // TODO: Commands are translated to strings so they can share a field with normal launch commands.
    // This should probably work better than this
    pub fn command(&self) -> String {
        match self {
            DefaultAppOption::WebBrowser => "WebBrowser".to_owned(),
            DefaultAppOption::FileManager => "FileManager".to_owned(),
            DefaultAppOption::MailClient => "MailClient".to_owned(),
            DefaultAppOption::Music => "Music".to_owned(),
            DefaultAppOption::Video => "Video".to_owned(),
            DefaultAppOption::Photos => "Photos".to_owned(),
            DefaultAppOption::Calendar => "Calendar".to_owned(),
            DefaultAppOption::Terminal => "Terminal".to_owned(),
            DefaultAppOption::TextEditor => "TextEditor".to_owned(),
        }
    }
    pub fn as_localized_string(&self) -> String {
        match self {
            DefaultAppOption::WebBrowser => fl!("web-browser"),
            DefaultAppOption::FileManager => fl!("file-manager"),
            DefaultAppOption::MailClient => fl!("mail-client"),
            DefaultAppOption::Music => fl!("music"),
            DefaultAppOption::Video => fl!("video"),
            DefaultAppOption::Photos => fl!("photos"),
            DefaultAppOption::Calendar => fl!("calendar"),
            DefaultAppOption::Terminal => fl!("terminal"),
            DefaultAppOption::TextEditor => fl!("text-editor"),
        }
    }
}

// Power action types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Hash)]
pub enum PowerActionOption {
    Lock,
    Logout,
    Suspend,
    Restart,
    Shutdown,
}
// localised option strings
impl PowerActionOption {
    // TODO: Commands are translated to strings so they can share a field with normal launch commands.
    // This should probably work better than this
    pub fn command(&self) -> String {
        match self {
            PowerActionOption::Lock => "Lock".to_owned(),
            PowerActionOption::Logout => "Logout".to_owned(),
            PowerActionOption::Suspend => "Suspend".to_owned(),
            PowerActionOption::Restart => "Restart".to_owned(),
            PowerActionOption::Shutdown => "Shutdown".to_owned(),
        }
    }
    pub fn as_localized_string(&self) -> String {
        match self {
            PowerActionOption::Lock => fl!("lock"),
            PowerActionOption::Logout => fl!("logout"),
            PowerActionOption::Suspend => fl!("suspend"),
            PowerActionOption::Restart => fl!("restart"),
            PowerActionOption::Shutdown => fl!("shutdown"),
        }
    }
}

// Individual menu item struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuItem {
    pub item_type: MenuItemType,
    pub label: Option<String>,
    pub command: Option<String>,
}
impl MenuItem {
    pub fn item_type(&self) -> MenuItemType {
        self.item_type.clone()
    }
    pub fn label(&self) -> Option<String> {
        self.label.clone()
    }
    pub fn command(&self) -> Option<String> {
        self.command.clone()
        /*
        match &self.item_type() {
            MenuItemType::DefaultAction => {
                let desktop_file = get_default_app(&self.command.clone().unwrap_or("".to_string()));

                match desktop_file {
                    Ok(result) => match parse_desktop_file(&result.unwrap_or("".to_string())) {
                        Ok(desktop_info) => Some(desktop_info.1),
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            }
            _ => self.command.clone(),
        }
        */
    }
    pub fn command_label(&self) -> Option<String> {
        let command = self.command.clone();
        match &self.item_type() {
            MenuItemType::DefaultAction => {
                let desktop_file = get_default_app(&self.command.clone().unwrap_or("".to_string()));
                match desktop_file {
                    Ok(result) => match parse_desktop_file(&result.unwrap_or("".to_string())) {
                        Ok(desktop_info) => Some(desktop_info.0),
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            }
            _ => self.command.clone(),
        }
    }
}

// Top level menu items struct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuItems {
    pub items: Vec<MenuItem>,
}
impl fmt::Display for MenuItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Default for MenuItems {
    fn default() -> Self {
        Self {
            items: vec![
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("about-system")),
                    command: Some(String::from("cosmic-settings about")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("applications")),
                    command: Some(String::from("cosmic-app-library")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("launcher")),
                    command: Some(String::from("cosmic-launcher")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("workspaces")),
                    command: Some(String::from("cosmic-workspaces")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("terminal")),
                    command: Some(String::from("cosmic-term")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("files")),
                    command: Some(String::from("cosmic-files")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("software")),
                    command: Some(String::from("cosmic-store")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("settings")),
                    command: Some(String::from("cosmic-settings")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(fl!("lock")),
                    command: Some(String::from("Lock")),
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(fl!("logout")),
                    command: Some(String::from("Logout")),
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(fl!("suspend")),
                    command: Some(String::from("Suspend")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(fl!("restart")),
                    command: Some(String::from("Restart")),
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(fl!("shutdown")),
                    command: Some(String::from("Shutdown")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(fl!("menu-settings").to_string()),
                    command: Some(String::from("cosmic-logomenu-settings")),
                },
            ],
        }
    }
}

pub fn is_flatpak() -> bool {
    env::var("FLATPAK_ID").is_ok()
        || Path::new("/.flatpak-info").exists()
        || env::var("container")
            .map(|v| v == "flatpak")
            .unwrap_or(false)
}

pub fn get_default_app(default_type: &str) -> Result<Option<String>, io::Error> {
    for mime in &DEFAULT_APP_MIMES[default_type] {
        let output = if is_flatpak() == true {
            Command::new("flatpak-spawn")
                .args(&["--host", "xdg-mime", "query", "default", mime])
                .output()?
        } else {
            Command::new("xdg-mime")
                .args(&["query", "default", mime])
                .output()?
        };

        if output.status.success() {
            let app = String::from_utf8_lossy(&output.stdout).trim().to_string();

            return Ok(Some(str::replace(&app, ".desktop", "")));
        }
    }

    Ok(None)
}

fn get_default_app_dir(default_type: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    for mime in &DEFAULT_APP_MIMES[default_type] {
        // Get the desktop file name
        let output = Command::new("flatpak-spawn")
            .arg("--host")
            .arg("xdg-mime")
            .arg("query")
            .arg("default")
            .arg(mime)
            .output()?;

        let desktop_file = String::from_utf8(output.stdout)?.trim().to_string();

        // Search for the desktop file including Flatpak directories
        let output = Command::new("flatpak-spawn")
            .arg("--host")
            .arg("sh")
            .arg("-c")
            .arg(format!(
                "find \
                /usr/share/applications \
                /usr/local/share/applications \
                ~/.local/share/applications \
                /var/lib/flatpak/exports/share/applications \
                ~/.local/share/flatpak/exports/share/applications \
                ~/.var/app/*/data/applications \
                -name '{}' 2>/dev/null | head -n 1",
                desktop_file
            ))
            .output()?;

        let path = String::from_utf8(output.stdout)?.trim().to_string();

        if !path.is_empty() {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn parse_desktop_file(app_desktop: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let locales = get_languages_from_env();
    let app_lower = app_desktop.to_lowercase();

    let entry = Iter::new(default_paths())
        .entries(Some(&locales))
        .find(|entry| entry.appid.to_lowercase() == app_lower)
        .ok_or_else(|| format!("Desktop entry for '{}' not found", app_desktop))?;

    // Use unwrap_or_default to avoid Option handling if appropriate for your use case
    let name = entry.name(&locales).unwrap().to_string();
    let exec = entry.exec().unwrap_or("").to_string();

    Ok((name, exec))
}

static DEFAULT_APP_MIMES: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    map.insert("WebBrowser", vec!["x-scheme-handler/http"]);
    map.insert("FileManager", vec!["inode/directory"]);
    map.insert("MailClient", vec!["x-scheme-handler/mailto"]);
    map.insert("Music", vec!["audio/mp3", "application/ogg", "video/mp4"]);
    map.insert("Video", vec!["video/mp4"]);
    map.insert("Photos", vec!["image/png"]);
    map.insert("Calendar", vec!["text/calendar"]);
    map.insert(
        "Terminal",
        vec![
            "x-scheme-handler/terminal",
            "application/x-terminal-emulator",
        ],
    );
    map.insert("TextEditor", vec!["text/plain"]);

    map
});

// Preload all logos
pub static IMAGES: phf::Map<&'static str, (&[u8], bool)> = phf_map! {
    "Aeryn" => (include_bytes!("../../res/icons/aeryn-logo.svg"), false),
    "Aeryn (Symbolic)" => (include_bytes!("../../res/icons/aeryn-logo-symbolic.svg"), true),
    "Alma" => (include_bytes!("../../res/icons/almalinux-logo.svg"), false),
    "Alma (Symbolic)" => (include_bytes!("../../res/icons/almalinux-logo-symbolic.svg"), true),
    "Alpine" => (include_bytes!("../../res/icons/alpine-logo.svg"), false),
    "Alpine (Symbolic)" => (include_bytes!("../../res/icons/alpine-logo-symbolic.svg"), true),
    "Arch" => (include_bytes!("../../res/icons/arch-logo.svg"), false),
    "Arch (Symbolic)" => (include_bytes!("../../res/icons/arch-logo-symbolic.svg"), true),
    "Asahi" => (include_bytes!("../../res/icons/asahilinux-logo.svg"), false),
    "Asahi (Symbolic)" => (include_bytes!("../../res/icons/asahilinux-logo-symbolic.svg"), true),
    "Bazzite" => (include_bytes!("../../res/icons/bazzite-logo.svg"), false),
    "Bazzite (Symbolic)" => (include_bytes!("../../res/icons/bazzite-logo-symbolic.svg"), true),
    "Cachy" => (include_bytes!("../../res/icons/cachy-logo.svg"), false),
    "Cachy (Symbolic)" => (include_bytes!("../../res/icons/cachy-logo-symbolic.svg"), true),
    "Cosmic (Black)" => (include_bytes!("../../res/icons/cosmic-logo-black.svg"), false),
    "Cosmic" => (include_bytes!("../../res/icons/cosmic-logo.svg"), false),
    "Cosmic (Symbolic)" => (include_bytes!("../../res/icons/cosmic-logo-symbolic.svg"), true),
    "Debian" => (include_bytes!("../../res/icons/debian-logo.svg"), false),
    "Debian (Symbolic)" => (include_bytes!("../../res/icons/debian-logo-symbolic.svg"), true),
    "EndeavourOS" => (include_bytes!("../../res/icons/endeavouros_logo.svg"), false),
    "EndeavourOS (Symbolic)" => (include_bytes!("../../res/icons/endeavouros_logo-symbolic.svg"), true),
    "Fedora" => (include_bytes!("../../res/icons/fedora-logo.svg"), false),
    "Fedora (Symbolic)" => (include_bytes!("../../res/icons/fedora-logo-symbolic.svg"), true),
    "Gentoo" => (include_bytes!("../../res/icons/gentoo-logo.svg"), false),
    "Gentoo (Symbolic)" => (include_bytes!("../../res/icons/gentoo-logo-symbolic.svg"), true),
    "Manjaro" => (include_bytes!("../../res/icons/manjaro-logo.svg"), false),
    "Manjaro (Symbolic)" => (include_bytes!("../../res/icons/manjaro-logo-symbolic.svg"), true),
    "MX (Symbolic)" => (include_bytes!("../../res/icons/mx-logo-symbolic.svg"), true),
    "NixOS" => (include_bytes!("../../res/icons/nixos-logo.svg"), false),
    "NixOS (Symbolic)" => (include_bytes!("../../res/icons/nixos-logo-symbolic.svg"), true),
    "Nobara (Symbolic)" => (include_bytes!("../../res/icons/nobara-logo-symbolic.svg"), true),
    "OpenSuse" => (include_bytes!("../../res/icons/opensuse-logo.svg"), false),
    "OpenSuse (Symbolic)" => (include_bytes!("../../res/icons/opensuse-logo-symbolic.svg"), true),
    "Pop!_OS" => (include_bytes!("../../res/icons/pop-os-logo.svg"), false),
    "Pop!_OS (Symbolic)" => (include_bytes!("../../res/icons/pop-os-logo-symbolic.svg"), true),
    "PostmarketOS" => (include_bytes!("../../res/icons/postmarket-logo.svg"), false),
    "PostmarketOS (Symbolic)" => (include_bytes!("../../res/icons/postmarket-logo-symbolic.svg"), true),
    "PureOS (Symbolic)" => (include_bytes!("../../res/icons/pureos-logo-symbolic.svg"), true),
    "Raspbian (Symbolic)" => (include_bytes!("../../res/icons/raspbian-logo-symbolic.svg"), true),
    "Red Hat" => (include_bytes!("../../res/icons/redhat-logo.svg"), false),
    "Red Hat (Symbolic)" => (include_bytes!("../../res/icons/redhat-logo-symbolic.svg"), true),
    "Redox (Symbolic)" => (include_bytes!("../../res/icons/redox-logo-symbolic.svg"), true),
    "Rocky" => (include_bytes!("../../res/icons/rockylinux-logo.svg"), false),
    "Rocky (Symbolic)" => (include_bytes!("../../res/icons/rockylinux-logo-symbolic.svg"), true),
    "Solus" => (include_bytes!("../../res/icons/solus-logo.svg"), false),
    "Solus (Symbolic)" => (include_bytes!("../../res/icons/solus-logo-symbolic.svg"), true),
    "SteamDeck (Orange)" => (include_bytes!("../../res/icons/steam-deck-le-logo.svg"), false),
    "SteamDeck (Blue)" => (include_bytes!("../../res/icons/steam-deck-logo.svg"), false),
    "SteamDeck (Symbolic)" => (include_bytes!("../../res/icons/steam-deck-logo-symbolic.svg"), true),
    "System76" => (include_bytes!("../../res/icons/system76-logo.svg"), false),
    "System76 (Symbolic)" => (include_bytes!("../../res/icons/system76-logo-symbolic.svg"), true),
    "Tux" => (include_bytes!("../../res/icons/tux-logo.svg"), false),
    "Tux (Symbolic)" => (include_bytes!("../../res/icons/tux-logo-symbolic.svg"), true),
    "Universal Blue" => (include_bytes!("../../res/icons/ublue-logo.svg"), false),
    "Universal Blue (Symbolic)" => (include_bytes!("../../res/icons/ublue-logo-symbolic.svg"), true),
    "Ubuntu" => (include_bytes!("../../res/icons/ubuntu-logo.svg"), false),
    "Ubuntu (Symbolic)" => (include_bytes!("../../res/icons/ubuntu-logo-symbolic.svg"), true),
    "Void" => (include_bytes!("../../res/icons/void-logo.svg"), false),
    "Void (Symbolic)" => (include_bytes!("../../res/icons/void-logo-symbolic.svg"), true),
    "Zorin" => (include_bytes!("../../res/icons/zorin-logo.svg"), false),
    "Zorin (Symbolic)" => (include_bytes!("../../res/icons/zorin-logo-symbolic.svg"), true),
};
