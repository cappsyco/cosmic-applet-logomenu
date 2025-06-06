// SPDX-License-Identifier: GPL-3.0-only

use crate::power;
use crate::power::PowerAction;
use cosmic::app::{Core, Task};
use cosmic::applet::{menu_button, padded_control};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::window::Id;
use cosmic::iced::{Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self};
use cosmic::{Application, Element};
use liblog::{IMAGES, LogoMenuConfig, MenuItemType};
use std::process::Command;

const ID: &str = "co.uk.cappsy.CosmicAppletLogoMenu";

pub struct LogoMenu {
    core: Core,
    popup: Option<Id>,
    config: LogoMenuConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    Run(String),
    Action(power::PowerAction),
    Zbus(Result<(), zbus::Error>),
    ConfigUpdate(LogoMenuConfig),
}

impl Application for LogoMenu {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Load config
        let helper = Config::new(ID, LogoMenuConfig::VERSION).ok();
        let config: LogoMenuConfig = helper
            .as_ref()
            .map(|helper| {
                LogoMenuConfig::get_entry(helper).unwrap_or_else(|(_errors, config)| config)
            })
            .unwrap_or_default();

        let app = LogoMenu {
            core,
            popup: None,
            config,
        };
        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        // Get the current logo with appropriate fallback
        let selected_logo_name = if IMAGES.contains_key(&self.config.logo) {
            &self.config.logo
        } else {
            &LogoMenuConfig::default().logo
        };
        let logo_bytes = IMAGES[selected_logo_name];

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

        // Get the menu from config
        let config_menuitems = &self.config.menu_items;

        let mut content_list = widget::column().padding([8, 0]).spacing(0);
        for item in &config_menuitems.items {
            match item.item_type() {
                MenuItemType::LaunchAction => {
                    content_list = content_list.push(
                        menu_button(widget::text::body(item.label().unwrap_or_default()))
                            .on_press(Message::Run(item.command().unwrap_or_default())),
                    )
                }
                MenuItemType::PowerAction => {
                    content_list = content_list.push(
                        menu_button(widget::text::body(item.label().unwrap_or_default())).on_press(
                            Message::Action(match item.command() {
                                Some(command) => match command.as_ref() {
                                    "Lock" => PowerAction::Lock,
                                    "LogOut" => PowerAction::LogOut,
                                    "Suspend" => PowerAction::Suspend,
                                    "Restart" => PowerAction::Restart,
                                    "Shutdown" => PowerAction::Shutdown,
                                    _ => PowerAction::Shutdown,
                                },
                                _ => PowerAction::Shutdown,
                            }),
                        ),
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

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            self.core
                .watch_config(ID)
                .map(|res| Message::ConfigUpdate(res.config)),
        ])
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
                };
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
                match Command::new("sh").arg("-c").arg(action).spawn() {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error executing command: {}", e),
                };
                return close_popup(self.popup);
            }
            Message::ConfigUpdate(config) => {
                self.config = config;
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

fn close_popup(mut popup: Option<Id>) -> Task<Message> {
    if let Some(p) = popup.take() {
        destroy_popup(p)
    } else {
        Task::none()
    }
}
