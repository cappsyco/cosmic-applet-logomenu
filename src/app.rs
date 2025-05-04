// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::applet::{menu_button, padded_control};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::window::Id;
use cosmic::iced::Limits;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self, settings};
use cosmic::{Application, Element};

use crate::fl;

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
#[derive(Default)]
pub struct YourApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<Id>,
    /// Example row toggler.
    example_row: bool,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    ToggleExampleRow(bool),
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for YourApp {
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

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = YourApp {
            core,
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let bytes = include_bytes!(concat!("../res/icons/ublue-logo-symbolic.svg"));

        self.core
            .applet
            .icon_button_from_handle(cosmic::widget::icon::from_svg_bytes(bytes).symbolic(true))
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let Spacing {
            space_xxs, space_s, ..
        } = cosmic::theme::active().cosmic().spacing;

        let mut content_list = widget::column().padding([8, 0]).spacing(0);

        content_list = content_list.push(
            menu_button(widget::text::body(fl!("app-library"))).on_press(Message::TogglePopup),
        );

        content_list = content_list.push(
            menu_button(widget::text::body(fl!("app-launcher"))).on_press(Message::TogglePopup),
        );

        content_list = content_list.push(
            menu_button(widget::text::body(fl!("workspaces"))).on_press(Message::TogglePopup),
        );

        content_list = content_list.push(
            padded_control(widget::divider::horizontal::default()).padding([space_xxs, space_s]),
        );

        content_list = content_list
            .push(menu_button(widget::text::body(fl!("software"))).on_press(Message::TogglePopup));

        content_list = content_list
            .push(menu_button(widget::text::body(fl!("terminal"))).on_press(Message::TogglePopup));

        content_list = content_list.push(
            menu_button(widget::text::body(fl!("containers"))).on_press(Message::TogglePopup),
        );

        content_list = content_list
            .push(menu_button(widget::text::body(fl!("system"))).on_press(Message::TogglePopup));

        content_list = content_list
            .push(
                padded_control(widget::divider::horizontal::default())
                    .padding([space_xxs, space_s]),
            )
            .push(
                menu_button(widget::text::body(fl!("menu-settings")))
                    .on_press(Message::TogglePopup),
            );

        self.core.applet.popup_container(content_list).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
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
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    /*
    #[macro_export]
    macro_rules! icon_handle {
        ($name:literal) => {{
            let bytes = include_bytes!(concat!("../res/icons/hicolor/16x16/", $name, ".svg"));
            cosmic::widget::icon::from_svg_bytes(bytes).symbolic(true)
        }};
    }
    */
}
