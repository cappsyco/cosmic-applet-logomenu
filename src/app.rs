// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::applet::{menu_button, padded_control};
use cosmic::cosmic_config::Config;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Limits};
use cosmic::iced_widget::row;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self, container, dropdown};
use cosmic::{Application, Element};
use phf::phf_map;
use std::process::Command;
use zbus::Connection;

use logind_zbus::{
    manager::ManagerProxy,
    session::{SessionClass, SessionProxy, SessionType},
    user::UserProxy,
};
use rustix::process::getuid;

pub mod cosmic_session;

use crate::config::{load_config, update_config};
use crate::fl;
use cosmic_session::CosmicSessionProxy;

const ID: &'static str = "co.uk.cappsy.CosmicAppletLogoMenu";
const CONFIG_VER: u64 = 1;

pub struct LogoMenu {
    core: Core,
    config: Config,
    popup: Option<Id>,
    show_menu_settings: bool,
    logo_options: Vec<String>,
    selected_logo_idx: Option<usize>,
    selected_logo_name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Run(String),
    UpdateLogo(usize),
    ToggleMenuSettings,
    Action(PowerAction),
    Zbus(Result<(), zbus::Error>),
}

#[derive(Debug, Clone, Copy)]
enum PowerAction {
    Lock,
    LogOut,
    Suspend,
    Restart,
    Shutdown,
}
impl PowerAction {
    fn perform(self) -> cosmic::iced::Task<cosmic::Action<Message>> {
        let msg = |m| cosmic::action::app(Message::Zbus(m));
        match self {
            PowerAction::Lock => cosmic::iced::Task::perform(lock(), msg),
            PowerAction::LogOut => cosmic::iced::Task::perform(log_out(), msg),
            PowerAction::Suspend => cosmic::iced::Task::perform(suspend(), msg),
            PowerAction::Restart => cosmic::iced::Task::perform(restart(), msg),
            PowerAction::Shutdown => cosmic::iced::Task::perform(shutdown(), msg),
        }
    }
}

impl Application for LogoMenu {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let default_logo = String::from("Cosmic (Symbolic)");

        let config_logo = match load_config("logo", CONFIG_VER) {
            Some(val) => val,
            None => default_logo.to_owned(),
        };

        let selected_logo_name = if LOGOS.contains_key(&config_logo) {
            config_logo
        } else {
            default_logo
        };

        let mut logo_options = vec![];
        for (key, _value) in &LOGOS {
            logo_options.push(key.to_string());
        }
        logo_options.sort();

        let selected_logo_idx = logo_options.iter().position(|n| n == &selected_logo_name);

        let app = LogoMenu {
            core,
            config: Config::new(ID, CONFIG_VER).unwrap(),
            popup: None,
            show_menu_settings: false,
            selected_logo_idx,
            selected_logo_name,
            logo_options,
        };
        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        let logo_bytes = LOGOS[&self.selected_logo_name];

        self.core
            .applet
            .icon_button_from_handle(
                cosmic::widget::icon::from_svg_bytes(logo_bytes.0).symbolic(logo_bytes.1),
            )
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let Spacing {
            space_xxs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        let menu_items = get_menu_items();
        let mut content_list = widget::column().padding([8, 0]).spacing(0);

        for item in menu_items {
            match item.item_type() {
                MenuItemType::TermAction => {
                    content_list = content_list.push(
                        menu_button(widget::text::body(match item.label() {
                            Some(label) => label,
                            None => String::from(""),
                        }))
                        .on_press(Message::Run(match item.exec() {
                            Some(exec) => exec,
                            None => String::from(""),
                        })),
                    )
                }
                MenuItemType::PowerAction => {
                    content_list = content_list.push(
                        menu_button(widget::text::body(match item.label() {
                            Some(label) => label,
                            None => String::from(""),
                        }))
                        .on_press(Message::Action(item.action().unwrap())),
                    )
                }
                MenuItemType::Divider => {
                    content_list = content_list.push(
                        padded_control(widget::divider::horizontal::default())
                            .padding([space_xxs, space_s]),
                    )
                }
            };
        }

        content_list = content_list.push(
            padded_control(widget::divider::horizontal::default()).padding([space_xxs, space_s]),
        );

        let dropdown_icon = if self.show_menu_settings {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        };
        let menu_settings_btn = menu_button(row![
            widget::text::body(fl!("menu-settings"))
                .width(Length::Fill)
                .height(Length::Fixed(24.0))
                .align_y(Alignment::Center),
            container(
                widget::icon::from_name(dropdown_icon)
                    .size(16)
                    .symbolic(true)
            )
            .center(Length::Fixed(24.0))
        ])
        .on_press(Message::ToggleMenuSettings);
        content_list = content_list.push(menu_settings_btn);

        if self.show_menu_settings {
            content_list = content_list.push(container(
                padded_control(dropdown(
                    &self.logo_options,
                    self.selected_logo_idx,
                    Message::UpdateLogo,
                ))
                .width(Length::Fill)
                .padding([space_xxs, space_s]),
            ));
        }

        self.core.applet.popup_container(content_list).into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::Action(action) => {
                match action {
                    PowerAction::LogOut => {
                        if let Err(_err) = Command::new("cosmic-osd").arg("log-out").spawn() {
                            //tracing::error!("Failed to spawn cosmic-osd. {err:?}");
                            return PowerAction::LogOut.perform();
                        }
                    }
                    PowerAction::Restart => {
                        if let Err(_err) = Command::new("cosmic-osd").arg("restart").spawn() {
                            //tracing::error!("Failed to spawn cosmic-osd. {err:?}");
                            return PowerAction::Restart.perform();
                        }
                    }
                    PowerAction::Shutdown => {
                        if let Err(_err) = Command::new("cosmic-osd").arg("shutdown").spawn() {
                            //tracing::error!("Failed to spawn cosmic-osd. {err:?}");
                            return PowerAction::Shutdown.perform();
                        }
                    }
                    a => return a.perform(),
                };

                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    Task::none()
                };
            }
            Message::Zbus(result) => {
                if let Err(e) = result {
                    eprintln!("cosmic-applet-power ERROR: '{}'", e);
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Run(action) => {
                let _ = Command::new("sh").arg("-c").arg(action).spawn().unwrap();
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    Task::none()
                };
            }
            Message::UpdateLogo(logo) => {
                self.selected_logo_name = self.logo_options[logo].clone();
                self.selected_logo_idx = Some(logo);

                if logo > 0 {
                    let _ = update_config(self.config.clone(), "logo", &self.selected_logo_name);
                }
            }
            Message::ToggleMenuSettings => {
                self.show_menu_settings = !self.show_menu_settings;
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

#[derive(Clone)]
pub enum MenuItemType {
    TermAction,
    PowerAction,
    Divider,
}
pub struct MenuItem {
    item_type: MenuItemType,
    label: Option<String>,
    exec: Option<String>,
    action: Option<PowerAction>,
}
impl MenuItem {
    pub fn item_type(&self) -> MenuItemType {
        self.item_type.clone()
    }
    pub fn label(&self) -> Option<String> {
        self.label.clone()
    }
    pub fn exec(&self) -> Option<String> {
        self.exec.clone()
    }
    pub fn action(&self) -> Option<PowerAction> {
        self.action.clone()
    }
}

pub fn get_menu_items() -> Vec<MenuItem> {
    let mut items = Vec::new();

    // Define menu items
    // TODO: Make this configurable
    items.push(MenuItem {
        item_type: MenuItemType::TermAction,
        label: Some(fl!("applications")),
        exec: Some(String::from("cosmic-app-library")),
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::TermAction,
        label: Some(fl!("launcher")),
        exec: Some(String::from("cosmic-launcher")),
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::TermAction,
        label: Some(fl!("workspaces")),
        exec: Some(String::from("cosmic-workspaces")),
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::Divider,
        label: None,
        exec: None,
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::TermAction,
        label: Some(fl!("terminal")),
        exec: Some(String::from("cosmic-term")),
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::TermAction,
        label: Some(fl!("files")),
        exec: Some(String::from("cosmic-files")),
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::TermAction,
        label: Some(fl!("software")),
        exec: Some(String::from("cosmic-store")),
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::Divider,
        label: None,
        exec: None,
        action: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("lock-screen")),
        exec: None,
        action: Some(PowerAction::Lock),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("log-out")),
        exec: None,
        action: Some(PowerAction::LogOut),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("suspend")),
        exec: None,
        action: Some(PowerAction::Suspend),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("restart")),
        exec: None,
        action: Some(PowerAction::Restart),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("shutdown")),
        exec: None,
        action: Some(PowerAction::Shutdown),
    });

    items
}

// ### System helpers

async fn restart() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let manager_proxy = ManagerProxy::new(&connection).await?;
    manager_proxy.reboot(true).await
}

async fn shutdown() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let manager_proxy = ManagerProxy::new(&connection).await?;
    manager_proxy.power_off(true).await
}

async fn suspend() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let manager_proxy = ManagerProxy::new(&connection).await?;
    manager_proxy.suspend(true).await
}

async fn lock() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let manager_proxy = ManagerProxy::new(&connection).await?;
    // Get the session this current process is running in
    let our_uid = getuid().as_raw() as u32;
    let user_path = manager_proxy.get_user(our_uid).await?;
    let user = UserProxy::builder(&connection)
        .path(user_path)?
        .build()
        .await?;
    // Lock all non-TTY sessions of this user
    let sessions = user.sessions().await?;
    let mut locked_successfully = false;
    for (_, session_path) in sessions {
        let Ok(session) = SessionProxy::builder(&connection)
            .path(session_path)?
            .build()
            .await
        else {
            continue;
        };

        if session.class().await == Ok(SessionClass::User)
            && session.type_().await? != SessionType::TTY
            && session.lock().await.is_ok()
        {
            locked_successfully = true;
        }
    }

    if locked_successfully {
        Ok(())
    } else {
        Err(zbus::Error::Failure("locking session failed".to_string()))
    }
}

async fn log_out() -> zbus::Result<()> {
    let connection = Connection::session().await?;
    let cosmic_session = CosmicSessionProxy::new(&connection).await?;
    cosmic_session.exit().await?;
    Ok(())
}

// Preload all logos
// TODO: Better way to do this?
static LOGOS: phf::Map<&'static str, (&[u8], bool)> = phf_map! {
    "Alma" => (include_bytes!("../res/icons/almalinux-logo.svg"), false),
    "Alma (Symbolic)" => (include_bytes!("../res/icons/almalinux-logo-symbolic.svg"), true),
    "Arch" => (include_bytes!("../res/icons/arch-logo.svg"), false),
    "Arch (Symbolic)" => (include_bytes!("../res/icons/arch-logo-symbolic.svg"), true),
    "Asahi" => (include_bytes!("../res/icons/asahilinux-logo.svg"), false),
    "Asahi (Symbolic)" => (include_bytes!("../res/icons/asahilinux-logo-symbolic.svg"), true),
    "Bazzite" => (include_bytes!("../res/icons/bazzite-logo.svg"), false),
    "Clear" => (include_bytes!("../res/icons/clear-linux-logo.svg"), false),
    "Cosmic (Black)" => (include_bytes!("../res/icons/cosmic-logo-black.svg"), false),
    "Cosmic" => (include_bytes!("../res/icons/cosmic-logo.svg"), false),
    "Cosmic (Symbolic)" => (include_bytes!("../res/icons/cosmic-logo-symbolic.svg"), true),
    "Debian" => (include_bytes!("../res/icons/debian-logo.svg"), false),
    "Debian (Symbolic)" => (include_bytes!("../res/icons/debian-logo-symbolic.svg"), true),
    "EndeavourOS" => (include_bytes!("../res/icons/endeavouros_logo.svg"), false),
    "EndeavourOS (Symbolic)" => (include_bytes!("../res/icons/endeavouros_logo-symbolic.svg"), true),
    "Fedora" => (include_bytes!("../res/icons/fedora-logo.svg"), false),
    "Fedora (Symbolic)" => (include_bytes!("../res/icons/fedora-logo-symbolic.svg"), true),
    "FreeBSD" => (include_bytes!("../res/icons/freebsd-logo.svg"), false),
    "FreeBSD (Symbolic)" => (include_bytes!("../res/icons/freebsd-logo-symbolic.svg"), true),
    "Garuda" => (include_bytes!("../res/icons/garuda-logo-symbolic.svg"), true),
    "Garuda (Symbolic)" => (include_bytes!("../res/icons/gentoo-logo.svg"), false),
    "Gentoo (Symbolic)" => (include_bytes!("../res/icons/gentoo-logo-symbolic.svg"), true),
    "Kali" => (include_bytes!("../res/icons/kali-linux-logo.svg"), false),
    "Kali (Symbolic)" => (include_bytes!("../res/icons/kali-linux-logo-symbolic.svg"), true),
    "Manjaro" => (include_bytes!("../res/icons/manjaro-logo.svg"), false),
    "Manjaro (Symbolic)" => (include_bytes!("../res/icons/manjaro-logo-symbolic.svg"), true),
    "MX (Symbolic)" => (include_bytes!("../res/icons/mx-logo-symbolic.svg"), true),
    "NetBSD" => (include_bytes!("../res/icons/netbsd-logo.svg"), false),
    "NixOS" => (include_bytes!("../res/icons/nixos-logo.svg"), false),
    "NixOS (Symbolic)" => (include_bytes!("../res/icons/nixos-logo-symbolic.svg"), true),
    "Nobara (Symbolic)" => (include_bytes!("../res/icons/nobara-logo-symbolic.svg"), true),
    "OpenBSD" => (include_bytes!("../res/icons/openbsd-logo.svg"), false),
    "OpenSuse" => (include_bytes!("../res/icons/opensuse-logo.svg"), false),
    "OpenSuse (Symbolic)" => (include_bytes!("../res/icons/opensuse-logo-symbolic.svg"), true),
    "Pop!_OS" => (include_bytes!("../res/icons/pop-os-logo.svg"), false),
    "Pop!_OS (Symbolic)" => (include_bytes!("../res/icons/pop-os-logo-symbolic.svg"), true),
    "PureOS (Symbolic)" => (include_bytes!("../res/icons/pureos-logo-symbolic.svg"), true),
    "Raspbian (Symbolic)" => (include_bytes!("../res/icons/raspbian-logo-symbolic.svg"), true),
    "Red Hat" => (include_bytes!("../res/icons/redhat-logo.svg"), false),
    "Red Hat (Symbolic)" => (include_bytes!("../res/icons/redhat-logo-symbolic.svg"), true),
    "Rocky" => (include_bytes!("../res/icons/rockylinux-logo.svg"), false),
    "Rocky (Symbolic)" => (include_bytes!("../res/icons/rockylinux-logo-symbolic.svg"), true),
    "ShastraOS" => (include_bytes!("../res/icons/shastraos-logo.svg"), false),
    "ShastraOS (Symbolic)" => (include_bytes!("../res/icons/shastraos-logo-symbolic.svg"), true),
    "Solus" => (include_bytes!("../res/icons/solus-logo.svg"), false),
    "Solus (Symbolic)" => (include_bytes!("../res/icons/solus-logo-symbolic.svg"), true),
    "SteamDeck (Orange)" => (include_bytes!("../res/icons/steam-deck-le-logo.svg"), false),
    "SteamDeck (Blue)" => (include_bytes!("../res/icons/steam-deck-logo.svg"), false),
    "SteamDeck (Symbolic)" => (include_bytes!("../res/icons/steam-deck-logo-symbolic.svg"), true),
    "Tux" => (include_bytes!("../res/icons/tux-logo.svg"), false),
    "Tux (Symbolic)" => (include_bytes!("../res/icons/tux-logo-symbolic.svg"), true),
    "uBlue" => (include_bytes!("../res/icons/ublue-logo.svg"), false),
    "uBlue (Symbolic)" => (include_bytes!("../res/icons/ublue-logo-symbolic.svg"), true),
    "Ubuntu" => (include_bytes!("../res/icons/ubuntu-logo.svg"), false),
    "Ubuntu (Symbolic)" => (include_bytes!("../res/icons/ubuntu-logo-symbolic.svg"), true),
    "Vanilla" => (include_bytes!("../res/icons/vanilla-logo.svg"), false),
    "Void" => (include_bytes!("../res/icons/void-logo.svg"), false),
    "Void (Symbolic)" => (include_bytes!("../res/icons/void-logo-symbolic.svg"), true),
    "Voyager (Symbolic)" => (include_bytes!("../res/icons/voyager-logo-symbolic.svg"), true),
    "Zorin" => (include_bytes!("../res/icons/zorin-logo.svg"), false),
    "Zorin (Symbolic)" => (include_bytes!("../res/icons/zorin-logo-symbolic.svg"), true),
};
