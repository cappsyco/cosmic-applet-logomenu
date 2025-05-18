// SPDX-License-Identifier: GPL-3.0-only

use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// Menu item types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MenuItemType {
    LaunchAction,
    PowerAction,
    Divider,
}

// Individual menu item struct
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    }
}

// Top lebel menu items struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItems {
    pub items: Vec<MenuItem>,
}
impl Display for MenuItems {
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
                    label: Some(String::from("About your system")),
                    command: Some(String::from("cosmic-settings about")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Applications")),
                    command: Some(String::from("applications")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Launcher")),
                    command: Some(String::from("cosmic-launcher")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Workspaces")),
                    command: Some(String::from("cosmic-workspaces")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Terminal")),
                    command: Some(String::from("cosmic-term")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Files")),
                    command: Some(String::from("cosmic-files")),
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Software")),
                    command: Some(String::from("cosmic-store")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(String::from("Lock")),
                    command: Some(String::from("Lock")),
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(String::from("Log out")),
                    command: Some(String::from("LogOut")),
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(String::from("Suspend")),
                    command: Some(String::from("Suspend")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(String::from("Restart")),
                    command: Some(String::from("Restart")),
                },
                MenuItem {
                    item_type: MenuItemType::PowerAction,
                    label: Some(String::from("Shutdown")),
                    command: Some(String::from("Shutdown")),
                },
                MenuItem {
                    item_type: MenuItemType::Divider,
                    label: None,
                    command: None,
                },
                MenuItem {
                    item_type: MenuItemType::LaunchAction,
                    label: Some(String::from("Menu settings...")),
                    command: Some(String::from("cosmic-logomenu-settings")),
                },
            ],
        }
    }
}

// Preload all logos
pub static IMAGES: phf::Map<&'static str, (&[u8], bool)> = phf_map! {
    "Aeryn" => (include_bytes!("../../res/icons/aeryn-logo.svg"), false),
    "Aeryn (Symbolic)" => (include_bytes!("../../res/icons/aeryn-logo-symbolic.svg"), true),
    "Alma" => (include_bytes!("../../res/icons/almalinux-logo.svg"), false),
    "Alma (Symbolic)" => (include_bytes!("../../res/icons/almalinux-logo-symbolic.svg"), true),
    "Arch" => (include_bytes!("../../res/icons/arch-logo.svg"), false),
    "Arch (Symbolic)" => (include_bytes!("../../res/icons/arch-logo-symbolic.svg"), true),
    "Asahi" => (include_bytes!("../../res/icons/asahilinux-logo.svg"), false),
    "Asahi (Symbolic)" => (include_bytes!("../../res/icons/asahilinux-logo-symbolic.svg"), true),
    "Bazzite" => (include_bytes!("../../res/icons/bazzite-logo.svg"), false),
    "Cachy" => (include_bytes!("../../res/icons/cachy-logo.svg"), false),
    "Cachy (Symbolic)" => (include_bytes!("../../res/icons/cachy-logo-symbolic.svg"), true),
    "Clear" => (include_bytes!("../../res/icons/clear-linux-logo.svg"), false),
    "Cosmic (Black)" => (include_bytes!("../../res/icons/cosmic-logo-black.svg"), false),
    "Cosmic" => (include_bytes!("../../res/icons/cosmic-logo.svg"), false),
    "Cosmic (Symbolic)" => (include_bytes!("../../res/icons/cosmic-logo-symbolic.svg"), true),
    "Debian" => (include_bytes!("../../res/icons/debian-logo.svg"), false),
    "Debian (Symbolic)" => (include_bytes!("../../res/icons/debian-logo-symbolic.svg"), true),
    "EndeavourOS" => (include_bytes!("../../res/icons/endeavouros_logo.svg"), false),
    "EndeavourOS (Symbolic)" => (include_bytes!("../../res/icons/endeavouros_logo-symbolic.svg"), true),
    "Fedora" => (include_bytes!("../../res/icons/fedora-logo.svg"), false),
    "Fedora (Symbolic)" => (include_bytes!("../../res/icons/fedora-logo-symbolic.svg"), true),
    "Garuda" => (include_bytes!("../../res/icons/garuda-logo-symbolic.svg"), true),
    "Garuda (Symbolic)" => (include_bytes!("../../res/icons/gentoo-logo.svg"), false),
    "Gentoo (Symbolic)" => (include_bytes!("../../res/icons/gentoo-logo-symbolic.svg"), true),
    "Kali" => (include_bytes!("../../res/icons/kali-linux-logo.svg"), false),
    "Kali (Symbolic)" => (include_bytes!("../../res/icons/kali-linux-logo-symbolic.svg"), true),
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
    "PureOS (Symbolic)" => (include_bytes!("../../res/icons/pureos-logo-symbolic.svg"), true),
    "Raspbian (Symbolic)" => (include_bytes!("../../res/icons/raspbian-logo-symbolic.svg"), true),
    "Red Hat" => (include_bytes!("../../res/icons/redhat-logo.svg"), false),
    "Red Hat (Symbolic)" => (include_bytes!("../../res/icons/redhat-logo-symbolic.svg"), true),
    "Redox" => (include_bytes!("../../res/icons/redox-logo.svg"), false),
    "Redox (Symbolic)" => (include_bytes!("../../res/icons/redox-logo-symbolic.svg"), true),
    "Rocky" => (include_bytes!("../../res/icons/rockylinux-logo.svg"), false),
    "Rocky (Symbolic)" => (include_bytes!("../../res/icons/rockylinux-logo-symbolic.svg"), true),
    "Solus" => (include_bytes!("../../res/icons/solus-logo.svg"), false),
    "Solus (Symbolic)" => (include_bytes!("../../res/icons/solus-logo-symbolic.svg"), true),
    "SteamDeck (Orange)" => (include_bytes!("../../res/icons/steam-deck-le-logo.svg"), false),
    "SteamDeck (Blue)" => (include_bytes!("../../res/icons/steam-deck-logo.svg"), false),
    "SteamDeck (Symbolic)" => (include_bytes!("../../res/icons/steam-deck-logo-symbolic.svg"), true),
    "Tux" => (include_bytes!("../../res/icons/tux-logo.svg"), false),
    "Tux (Symbolic)" => (include_bytes!("../../res/icons/tux-logo-symbolic.svg"), true),
    "Universal Blue" => (include_bytes!("../../res/icons/ublue-logo.svg"), false),
    "Universal Blue (Symbolic)" => (include_bytes!("../../res/icons/ublue-logo-symbolic.svg"), true),
    "Ubuntu" => (include_bytes!("../../res/icons/ubuntu-logo.svg"), false),
    "Ubuntu (Symbolic)" => (include_bytes!("../../res/icons/ubuntu-logo-symbolic.svg"), true),
    "Vanilla" => (include_bytes!("../../res/icons/vanilla-logo.svg"), false),
    "Void" => (include_bytes!("../../res/icons/void-logo.svg"), false),
    "Void (Symbolic)" => (include_bytes!("../../res/icons/void-logo-symbolic.svg"), true),
    "Zorin" => (include_bytes!("../../res/icons/zorin-logo.svg"), false),
    "Zorin (Symbolic)" => (include_bytes!("../../res/icons/zorin-logo-symbolic.svg"), true),
};
