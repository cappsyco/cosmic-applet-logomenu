// SPDX-License-Identifier: GPL-3.0-only

use app::LogoMenu;
mod app;
mod config;
mod power;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<LogoMenu>(())
}
