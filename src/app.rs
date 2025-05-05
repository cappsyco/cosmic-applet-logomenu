// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::applet::{menu_button, padded_control};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::window::Id;
use cosmic::iced::Limits;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self};
use cosmic::{Application, Element};
use std::process::Command;

use crate::fl;

#[derive(Default)]
pub struct LogoMenu {
    core: Core,
    popup: Option<Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Run(String),
}

impl Application for LogoMenu {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "co.uk.cappsy.CosmicAppletLogoMenu";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = LogoMenu {
            core,
            ..Default::default()
        };
        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        let menu_icon = get_menu_icon();
        let icon_bytes = include_bytes!("../res/icons/cosmic-logo-symbolic.svg");

        self.core
            .applet
            .icon_button_from_handle(
                cosmic::widget::icon::from_svg_bytes(icon_bytes).symbolic(menu_icon.symbolic()),
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
                MenuItemType::Action => {
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
                MenuItemType::Divider => {
                    content_list = content_list.push(
                        padded_control(widget::divider::horizontal::default())
                            .padding([space_xxs, space_s]),
                    )
                }
            };
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
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

pub struct MenuIcon {
    symbolic: bool,
}
impl MenuIcon {
    pub fn symbolic(&self) -> bool {
        self.symbolic
    }
}

#[derive(Clone)]
pub enum MenuItemType {
    Action,
    Divider,
}
pub struct MenuItem {
    item_type: MenuItemType,
    label: Option<String>,
    exec: Option<String>,
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
}

pub fn get_menu_icon() -> MenuIcon {
    // Get the logo
    // TODO: Make configurable
    MenuIcon { symbolic: true }
}

pub fn get_menu_items() -> Vec<MenuItem> {
    let mut items = Vec::new();

    // Define menu items
    // TODO: Make this configurable
    items.push(MenuItem {
        item_type: MenuItemType::Action,
        label: Some(fl!("applications")),
        exec: Some(String::from("cosmic-app-library")),
    });
    items.push(MenuItem {
        item_type: MenuItemType::Action,
        label: Some(fl!("launcher")),
        exec: Some(String::from("cosmic-launcher")),
    });
    items.push(MenuItem {
        item_type: MenuItemType::Action,
        label: Some(fl!("workspaces")),
        exec: Some(String::from("cosmic-workspaces")),
    });
    items.push(MenuItem {
        item_type: MenuItemType::Divider,
        label: None,
        exec: None,
    });
    items.push(MenuItem {
        item_type: MenuItemType::Action,
        label: Some(fl!("terminal")),
        exec: Some(String::from("cosmic-term")),
    });
    items.push(MenuItem {
        item_type: MenuItemType::Action,
        label: Some(fl!("files")),
        exec: Some(String::from("cosmic-files")),
    });
    items.push(MenuItem {
        item_type: MenuItemType::Action,
        label: Some(fl!("software")),
        exec: Some(String::from("cosmic-store")),
    });

    items
}
