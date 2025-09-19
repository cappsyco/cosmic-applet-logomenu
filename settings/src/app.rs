// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{load_config, update_config};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::Config;
use cosmic::iced::{Alignment, Length, Radius};
use cosmic::iced_widget::{rule, scrollable};
use cosmic::prelude::*;
use cosmic::widget::{self, Space, container, dropdown, menu, settings, toggler};
use cosmic::{cosmic_theme, theme};
use liblog::fl;
use liblog::{IMAGES, MenuItem, MenuItemType, MenuItems, PowerActionOption};
use rfd::FileDialog;
use std::collections::{HashMap, VecDeque};
use std::path::Path;

const APP_ICON: &[u8] =
    include_bytes!("../../res/icons/hicolor/scalable/apps/co.uk.cappsy.CosmicAppletLogoMenu.svg");
const CONFIG_VER: u64 = 1;
const CONFIG_ID: &str = "co.uk.cappsy.CosmicAppletLogoMenu";

#[derive(Clone, Debug)]
pub enum DialogPage {
    EditItem(usize, MenuItem),
    RemoveItem(usize),
    ResetMenu,
}

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    dialog_pages: VecDeque<DialogPage>,

    logo_options: Vec<String>,
    selected_logo_idx: Option<usize>,
    selected_logo_name: String,
    custom_logo_active: bool,
    custom_logo_path: String,
    menu_items: Vec<MenuItem>,
    menu_types: Vec<MenuItemType>,
    menu_type_labels: Vec<String>,
    power_actions: Vec<PowerActionOption>,
    power_action_labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleContextPage(ContextPage),
    UpdateLogo(usize),
    ToggleCustomLogo(bool),
    UpdateCustomLogo,
    AddItem(MenuItemType),
    SaveItem(usize, MenuItem),
    RemoveItem(usize),
    MoveItem(OrderDirection, usize),
    ResetMenu,
    DialogUpdate(DialogPage),
    DialogCancel,
    DialogEditItem(usize, MenuItem),
    DialogRemoveItem(usize),
    DialogResetMenu,
}

#[derive(Debug, Clone)]
pub enum OrderDirection {
    Up,
    Down,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "co.uk.cappsy.CosmicAppletLogoMenu.Settings";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Get the current logo, with fallbacks to the default
        let default_logo = String::from("Cosmic (Symbolic)");
        let config_logo = match load_config("logo", CONFIG_VER) {
            Some(val) => val,
            None => default_logo.to_owned(),
        };
        let selected_logo_name = if IMAGES.contains_key(&config_logo) {
            config_logo
        } else {
            default_logo
        };

        // Break out logos into options for setting
        let mut logo_options = vec![];
        let images_iter = &IMAGES;
        for (key, _value) in images_iter {
            logo_options.push(key.to_string());
        }
        logo_options.sort();
        let selected_logo_idx = logo_options.iter().position(|n| n == &selected_logo_name);

        // Load in menu items from settings
        let menu_items = get_menu_items();

        // get custom logo status and path
        let custom_logo_active = match load_config("custom_logo_active", CONFIG_VER) {
            Some(val) => val,
            None => false,
        };
        let custom_logo_path = match load_config("custom_logo_path", CONFIG_VER) {
            Some(val) => val,
            None => "".to_owned(),
        };

        let menu_types = vec![MenuItemType::LaunchAction, MenuItemType::PowerAction];
        let menu_type_labels: Vec<String> =
            menu_types.iter().map(|t| t.as_localized_string()).collect();

        let power_actions = vec![
            PowerActionOption::Lock,
            PowerActionOption::Logout,
            PowerActionOption::Suspend,
            PowerActionOption::Restart,
            PowerActionOption::Shutdown,
        ];
        let power_action_labels: Vec<String> = power_actions
            .iter()
            .map(|t| t.as_localized_string())
            .collect();

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            config: Config::new(CONFIG_ID, CONFIG_VER).unwrap(),
            dialog_pages: VecDeque::new(),
            logo_options,
            selected_logo_idx,
            selected_logo_name,
            menu_items,
            custom_logo_active,
            custom_logo_path,
            menu_types,
            menu_type_labels,
            power_actions,
            power_action_labels,
        };

        let command = app.update_title();
        (app, command)
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    fn view(&self) -> Element<'_, Self::Message> {
        //  Get theme info
        let theme = cosmic::theme::active();
        let padding = if self.core.is_condensed() {
            theme.cosmic().space_s()
        } else {
            theme.cosmic().space_l()
        };

        // Start container
        let mut page_content = widget::column()
            .padding(0.)
            .width(Length::Fill)
            .align_x(Alignment::Start);

        // Title
        page_content = page_content.push(
            widget::row().push(
                widget::column()
                    .push(widget::text::title3(fl!("app-title")))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            ),
        );
        page_content = page_content.push(Space::with_height(padding));

        // Set currently selected logo
        let logo_widget =
            if self.custom_logo_active == true && Path::new(&self.custom_logo_path).exists() {
                widget::svg(widget::svg::Handle::from_path(&self.custom_logo_path))
                    .symbolic(false)
                    .width(150)
            } else {
                let logo_bytes = IMAGES[&self.selected_logo_name];
                widget::svg(widget::svg::Handle::from_memory(logo_bytes.0))
                    .symbolic(logo_bytes.1)
                    .width(150)
            };

        // Display logo header
        page_content = page_content.push(
            widget::row().push(
                widget::column()
                    .push(logo_widget)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            ),
        );
        page_content = page_content.push(Space::with_height(padding));

        // Menu settings
        let mut menu_settings = settings::section().add({
            Element::from(
                settings::item::builder(fl!("use-custom-logo")).control(
                    toggler(self.custom_logo_active)
                        .on_toggle(|value| Message::ToggleCustomLogo(value)),
                ),
            )
        });
        if self.custom_logo_active == true {
            let file_name = if &self.custom_logo_path != "" {
                Path::new(&self.custom_logo_path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap_or("")
            } else {
                ""
            };
            menu_settings = menu_settings.add({
                Element::from(
                    settings::item::builder(file_name).control(
                        widget::button::standard(fl!("select-custom-logo"))
                            .on_press(Message::UpdateCustomLogo),
                    ),
                )
            });
        } else {
            menu_settings = menu_settings.add({
                Element::from(settings::item::builder(fl!("logo")).control(dropdown(
                    &self.logo_options,
                    self.selected_logo_idx,
                    Message::UpdateLogo,
                )))
            })
        }

        page_content = page_content.push(menu_settings);
        page_content = page_content.push(Space::with_height(padding));

        // Add buttons
        page_content = page_content.push(
            container(
                widget::row::with_capacity(3)
                    .push(
                        widget::button::suggested(fl!("add-menu-item"))
                            .on_press(Message::AddItem(MenuItemType::LaunchAction))
                            .apply(Element::from),
                    )
                    .push(
                        widget::button::standard(fl!("add-divider"))
                            .on_press(Message::AddItem(MenuItemType::Divider))
                            .apply(Element::from),
                    )
                    .push(
                        widget::button::destructive(fl!("reset-to-default"))
                            .on_press(Message::DialogResetMenu)
                            .apply(Element::from),
                    )
                    .spacing(10)
                    .apply(Element::from),
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),
        );
        page_content = page_content.push(Space::with_height(15));

        // Menu builder
        let mut menu_item_controls = settings::section();
        let menu_items = &self.menu_items;

        for (i, menu_item) in menu_items.iter().enumerate() {
            let mut menu_item_row = widget::row().push(
                widget::row::with_capacity(2)
                    .push(
                        widget::button::icon(widget::icon::from_name("pan-up-symbolic"))
                            .on_press(Message::MoveItem(OrderDirection::Up, i)),
                    )
                    .push(
                        widget::button::icon(widget::icon::from_name("pan-down-symbolic"))
                            .on_press(Message::MoveItem(OrderDirection::Down, i)),
                    ),
            );

            // item icon if not Divider
            if menu_item.item_type() != MenuItemType::Divider {
                menu_item_row = menu_item_row.push(
                    container(widget::icon::from_name(match menu_item.item_type() {
                        MenuItemType::LaunchAction => "utilities-terminal-symbolic",
                        MenuItemType::PowerAction => "system-shutdown-symbolic",
                        _ => "",
                    }))
                    .padding([8, 15, 0, 10]),
                )
            }

            // item label and controls
            menu_item_row = menu_item_row
                .push(match menu_item.label() {
                    Some(label) => {
                        let mut label_string = label;
                        let command_string = menu_item.command().unwrap_or_default();

                        if command_string != "" {
                            label_string.push_str("   ::   ");
                            label_string.push_str(&command_string);
                        }

                        container(cosmic::widget::text(label_string))
                            .width(Length::Fill)
                            .padding([5, 10, 0, 0])
                    }
                    _ => container(widget::divider::horizontal::default().class(
                        theme::Rule::custom(move |theme| {
                            let cosmic = theme.cosmic();
                            let divider_color = &cosmic.on_primary_component_color();

                            rule::Style {
                                color: cosmic::iced::Color::from_rgb(
                                    divider_color.red,
                                    divider_color.green,
                                    divider_color.blue,
                                ),
                                width: 1,
                                radius: Radius::new(0),
                                fill_mode: rule::FillMode::Full,
                            }
                        }),
                    ))
                    .padding([15, 10]),
                })
                .push(
                    widget::row::with_capacity(2)
                        .push(
                            widget::button::icon(widget::icon::from_name("edit-symbolic"))
                                .on_press_maybe(match menu_item.item_type() {
                                    MenuItemType::Divider => None,
                                    _ => Some(Message::DialogEditItem(i, menu_item.clone())),
                                }),
                        )
                        .push(
                            widget::button::icon(widget::icon::from_name("edit-delete-symbolic"))
                                .on_press(Message::DialogRemoveItem(i)),
                        ),
                );

            // apply row to list
            menu_item_controls = menu_item_controls.add(cosmic::Element::from(menu_item_row));
        }
        page_content = page_content.push(menu_item_controls);
        page_content = page_content.push(Space::with_height(15));

        // TODO: This works for now but it needs to be moved away
        // from the view function so it only triggers when needed.
        update_config(
            self.config.clone(),
            "menu_items",
            MenuItems {
                items: self.menu_items.clone(),
            },
        );

        // Combine all elements to finished page
        let page_container = container(page_content)
            .max_width(600)
            .width(Length::Fill)
            .apply(container)
            .center_x(Length::Fill)
            .padding([0, padding]);

        // Display
        let content: Element<_> = scrollable(page_container).into();

        content
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        let dialog_page = self.dialog_pages.front()?;

        let dialog = match dialog_page {
            DialogPage::EditItem(i, menu_item) => {
                let label = menu_item.label().unwrap_or_default();
                let command = menu_item.command().unwrap_or_default();
                let item_type = menu_item.item_type();

                let type_input = {
                    let menu_types = self.menu_types.clone();
                    let selected_type = self
                        .menu_types
                        .iter()
                        .position(|&r| r == item_type)
                        .unwrap_or(0);
                    let menu_item = menu_item.clone();
                    let i = *i;

                    widget::container(
                        widget::row::with_capacity(2)
                            .push(
                                widget::text(fl!("type"))
                                    .align_y(Alignment::Center)
                                    .height(30)
                                    .width(120),
                            )
                            .push(
                                dropdown(
                                    &self.menu_type_labels,
                                    Some(selected_type),
                                    move |value| {
                                        let mut command = None;
                                        if menu_types[value] == MenuItemType::PowerAction {
                                            command = Some(String::from("Lock"));
                                        }
                                        Message::DialogUpdate(DialogPage::EditItem(
                                            i,
                                            MenuItem {
                                                item_type: menu_types[value],
                                                command,
                                                ..menu_item.clone()
                                            },
                                        ))
                                    },
                                )
                                .width(Length::Fill),
                            ),
                    )
                };

                let label_input = widget::container(
                    widget::row::with_capacity(2)
                        .push(
                            widget::text(fl!("label"))
                                .align_y(Alignment::Center)
                                .height(30)
                                .width(120),
                        )
                        .push(
                            widget::text_input("", label.clone())
                                .on_input(move |value| {
                                    Message::DialogUpdate(DialogPage::EditItem(
                                        *i,
                                        MenuItem {
                                            label: Some(value),
                                            ..menu_item.clone()
                                        },
                                    ))
                                })
                                .width(Length::Fill),
                        ),
                );

                let command_input = if item_type == MenuItemType::PowerAction {
                    let power_actions = self.power_actions.clone();
                    let selected_power_action = self
                        .power_actions
                        .iter()
                        .position(|r| *r.command() == command)
                        .unwrap_or(0);
                    let menu_item = menu_item.clone();
                    let i = *i;

                    widget::container(
                        widget::row::with_capacity(2)
                            .push(
                                widget::text(fl!("command"))
                                    .align_y(Alignment::Center)
                                    .height(30)
                                    .width(120),
                            )
                            .push(
                                dropdown(
                                    &self.power_action_labels,
                                    Some(selected_power_action),
                                    move |value| {
                                        Message::DialogUpdate(DialogPage::EditItem(
                                            i,
                                            MenuItem {
                                                command: Some(power_actions[value].command()),
                                                ..menu_item.clone()
                                            },
                                        ))
                                    },
                                )
                                .width(Length::Fill),
                            ),
                    )
                } else {
                    widget::container(
                        widget::row::with_capacity(2)
                            .push(
                                widget::text(fl!("command"))
                                    .align_y(Alignment::Center)
                                    .height(30)
                                    .width(120),
                            )
                            .push(
                                widget::text_input("", command.clone())
                                    .on_input(|value| {
                                        Message::DialogUpdate(DialogPage::EditItem(
                                            *i,
                                            MenuItem {
                                                command: Some(value),
                                                ..menu_item.clone()
                                            },
                                        ))
                                    })
                                    .width(Length::Fill),
                            ),
                    )
                };

                // validation
                let complete_maybe = if label.is_empty() {
                    None
                } else {
                    Some(Message::SaveItem(*i, menu_item.clone()))
                };

                let save_button = widget::button::suggested(fl!("save"))
                    .on_press_maybe(complete_maybe)
                    .apply(Element::from);

                let cancel_button =
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel);

                widget::dialog()
                    .title(fl!("edit-menu-item"))
                    .control(
                        widget::ListColumn::default()
                            .add(type_input)
                            .add(label_input)
                            .add(command_input),
                    )
                    .primary_action(save_button)
                    .secondary_action(cancel_button)
                    .apply(Element::from)
            }

            DialogPage::RemoveItem(i) => widget::dialog()
                .title(fl!("remove-item"))
                .primary_action(
                    widget::button::suggested(fl!("remove")).on_press(Message::RemoveItem(*i)),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .apply(Element::from),

            DialogPage::ResetMenu => widget::dialog()
                .title(fl!("reset-to-default"))
                .primary_action(
                    widget::button::destructive(fl!("reset")).on_press(Message::ResetMenu),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                )
                .apply(Element::from),
        };

        Some(dialog)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateLogo(logo) => {
                self.selected_logo_name = self.logo_options[logo].clone();
                self.selected_logo_idx = Some(logo);

                update_config(self.config.clone(), "logo", &self.selected_logo_name);
            }

            Message::ToggleCustomLogo(toggle) => {
                self.custom_logo_active = toggle;

                update_config(
                    self.config.clone(),
                    "custom_logo_active",
                    &self.custom_logo_active,
                );
            }

            Message::UpdateCustomLogo => {
                let file = FileDialog::new()
                    .add_filter("svg", &["svg"])
                    .set_directory("~/")
                    .pick_file();

                match file {
                    Some(path) => {
                        let path_string = path.to_str().unwrap_or("");
                        update_config(self.config.clone(), "custom_logo_path", &path_string);
                        self.custom_logo_path = path_string.to_owned();
                    }
                    None => {}
                };
            }

            Message::AddItem(item_type) => {
                let new_item = MenuItem {
                    item_type: item_type.clone(),
                    label: match &item_type {
                        MenuItemType::LaunchAction => Some(fl!("new-launcher")),
                        _ => None,
                    },
                    command: match &item_type {
                        MenuItemType::LaunchAction => {
                            Some(String::from("cosmic-logomenu-settings"))
                        }
                        _ => None,
                    },
                };
                self.menu_items.splice(0..0, vec![new_item.clone()]);

                if item_type == MenuItemType::LaunchAction {
                    self.dialog_pages
                        .push_front(DialogPage::EditItem(0, new_item.clone()));
                }
            }

            Message::DialogUpdate(dialog_page) => {
                if !self.dialog_pages.is_empty() {
                    self.dialog_pages[0] = dialog_page;
                }
            }

            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }

            Message::DialogEditItem(i, menu_item) => {
                self.dialog_pages
                    .push_front(DialogPage::EditItem(i, menu_item));
            }

            Message::DialogRemoveItem(i) => {
                self.dialog_pages.push_front(DialogPage::RemoveItem(i));
            }

            Message::SaveItem(i, menu_item) => {
                self.menu_items[i] = menu_item;
                self.dialog_pages.pop_front();
            }

            Message::RemoveItem(i) => {
                self.menu_items.remove(i);
                self.dialog_pages.pop_front();
            }

            Message::MoveItem(dir, i) => {
                let j = match dir {
                    OrderDirection::Up => {
                        if i == 0 {
                            i
                        } else {
                            i - 1
                        }
                    }
                    OrderDirection::Down => {
                        if i == self.menu_items.len() - 1 {
                            i
                        } else {
                            i + 1
                        }
                    }
                };

                if i != j {
                    let a = self.menu_items[i].clone();
                    let b = self.menu_items[j].clone();
                    self.menu_items[j] = a;
                    self.menu_items[i] = b;
                }
            }

            Message::DialogResetMenu => {
                self.dialog_pages.push_front(DialogPage::ResetMenu);
            }

            Message::ResetMenu => {
                self.menu_items = MenuItems::default().items;
                self.dialog_pages.pop_front();
            }
        }
        Task::none()
    }
}

impl AppModel {
    pub fn about(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON)).width(120);
        let title = widget::text::title3(fl!("app-title"));

        widget::column()
            .push(icon)
            .push(Space::with_height(10))
            .push(title)
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .width(Length::Fill)
            .into()
    }

    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(fl!("app-title"), id)
        } else {
            Task::none()
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

pub fn get_menu_items() -> Vec<MenuItem> {
    let mut display_items = Vec::new();

    // Get the menu with a fallback to default if invalid or missing
    let config_menuitems: MenuItems = load_config("menu_items", CONFIG_VER).unwrap_or_default();

    for menuitem in config_menuitems.items {
        display_items.push(menuitem);
    }

    display_items
}
