// SPDX-License-Identifier: GPL-3.0-only

use crate::config::load_config;
use crate::fl;
use crate::logos;
use crate::power;
use cosmic::app::{Core, Task};
use cosmic::applet::{menu_button, padded_control};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Limits};
use cosmic::iced_widget::row;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self};
use cosmic::{Application, Element};
use std::process::Command;

const ID: &'static str = "co.uk.cappsy.CosmicAppletLogoMenu";
const CONFIG_VER: u64 = 1;

pub struct LogoMenu {
    core: Core,
    popup: Option<Id>,
    selected_logo_name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Run(String),
    Action(power::PowerAction),
    Zbus(Result<(), zbus::Error>),
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

        let selected_logo_name = if logos::IMAGES.contains_key(&config_logo) {
            config_logo
        } else {
            default_logo
        };

        let app = LogoMenu {
            core,
            popup: None,
            selected_logo_name,
        };
        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        let logo_bytes = logos::IMAGES[&self.selected_logo_name];

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

        let menu_settings_btn = menu_button(row![widget::text::body(fl!("menu-settings"))
            .width(Length::Fill)
            .height(Length::Fixed(24.0))
            .align_y(Alignment::Center)])
        .on_press(Message::Run(String::from("cosmic-logomenu-settings")));
        content_list = content_list.push(menu_settings_btn);

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
                    power::PowerAction::LogOut => {
                        if let Err(_err) = Command::new("cosmic-osd").arg("log-out").spawn() {
                            //tracing::error!("Failed to spawn cosmic-osd. {err:?}");
                            return power::PowerAction::LogOut.perform();
                        }
                    }
                    power::PowerAction::Restart => {
                        if let Err(_err) = Command::new("cosmic-osd").arg("restart").spawn() {
                            //tracing::error!("Failed to spawn cosmic-osd. {err:?}");
                            return power::PowerAction::Restart.perform();
                        }
                    }
                    power::PowerAction::Shutdown => {
                        if let Err(_err) = Command::new("cosmic-osd").arg("shutdown").spawn() {
                            //tracing::error!("Failed to spawn cosmic-osd. {err:?}");
                            return power::PowerAction::Shutdown.perform();
                        }
                    }
                    a => return a.perform(),
                };

                return close_popup(self.popup);
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
                return close_popup(self.popup);
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

fn close_popup(mut popup: Option<Id>) -> Task<Message> {
    return if let Some(p) = popup.take() {
        destroy_popup(p)
    } else {
        Task::none()
    };
}

// TODO: Break out the rest of this file into its own module, with:
// * hide / show toggles for default actions
// * renaming for default actions
// * custom options / dividers
// * reording of option
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
    action: Option<power::PowerAction>,
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
    pub fn action(&self) -> Option<power::PowerAction> {
        self.action.clone()
    }
}

pub fn get_menu_items() -> Vec<MenuItem> {
    let mut items = Vec::new();

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
        action: Some(power::PowerAction::Lock),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("log-out")),
        exec: None,
        action: Some(power::PowerAction::LogOut),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("suspend")),
        exec: None,
        action: Some(power::PowerAction::Suspend),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("restart")),
        exec: None,
        action: Some(power::PowerAction::Restart),
    });
    items.push(MenuItem {
        item_type: MenuItemType::PowerAction,
        label: Some(fl!("shutdown")),
        exec: None,
        action: Some(power::PowerAction::Shutdown),
    });

    items
}
