// SPDX-License-Identifier: MPL-2.0

use crate::config::{load_config, update_config};
use crate::fl;
use crate::logos;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::Config;
use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::scrollable;
use cosmic::prelude::*;
use cosmic::widget::{self, container, dropdown, menu, settings, Space};
use cosmic::{cosmic_theme, theme};
use std::collections::HashMap;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");
const CONFIG_VER: u64 = 1;
const CONFIG_ID: &'static str = "co.uk.cappsy.CosmicAppletLogoMenu";

pub struct AppModel {
    core: cosmic::Core,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: Config,
    logo_options: Vec<String>,
    selected_logo_idx: Option<usize>,
    selected_logo_name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    ToggleContextPage(ContextPage),
    LaunchUrl(String),
    UpdateLogo(usize),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "co.uk.cappsy.CosmicLogoMenuSettings";

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

        let mut logo_options = vec![];
        let images_iter = &logos::IMAGES;
        for (key, _value) in images_iter {
            logo_options.push(key.to_string());
        }
        logo_options.sort();

        let selected_logo_idx = logo_options.iter().position(|n| n == &selected_logo_name);

        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            config: Config::new(CONFIG_ID, CONFIG_VER).unwrap(),
            logo_options,
            selected_logo_idx,
            selected_logo_name,
        };

        let command = app.update_title();
        (app, command)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
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

    fn view(&self) -> Element<Self::Message> {
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
                    .push(widget::text::title3("COSMIC Logo Menu - Settings"))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            ),
        );
        page_content = page_content.push(Space::with_height(30));

        // Currently selected logo
        let logo_bytes = logos::IMAGES[&self.selected_logo_name];
        page_content = page_content.push(
            widget::row().push(
                widget::column()
                    .push(
                        widget::svg(widget::svg::Handle::from_memory(logo_bytes.0))
                            .symbolic(logo_bytes.1)
                            .width(75),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            ),
        );

        // Logo selector
        page_content = page_content.push(settings::section().title("Logo").add({
            cosmic::Element::from(settings::item::builder("Selected logo").control(dropdown(
                &self.logo_options,
                self.selected_logo_idx,
                Message::UpdateLogo,
            )))
        }));

        // Combine all elements to finished page
        let page_container = scrollable(
            container(page_content)
                .max_width(600)
                .width(Length::Fill)
                .apply(container)
                .center_x(Length::Fill)
                .padding([0, padding]),
        );

        // Display
        let content: Element<_> = container(page_container)
            .padding([cosmic::theme::active().cosmic().space_xxs(), 0])
            .into();

        content
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },

            Message::UpdateLogo(logo) => {
                self.selected_logo_name = self.logo_options[logo].clone();
                self.selected_logo_idx = Some(logo);

                if logo > 0 {
                    let _ = update_config(self.config.clone(), "logo", &self.selected_logo_name);
                }
            }
        }
        Task::none()
    }
}

impl AppModel {
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));
        let title = widget::text::title3(fl!("app-title"));

        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
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
