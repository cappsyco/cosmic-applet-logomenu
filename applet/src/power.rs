// SPDX-License-Identifier: GPL-3.0-only
//
// system helpers from cosmic-applet-power:
// https://github.com/pop-os/cosmic-applets

pub mod cosmic_session;

use crate::app;
use cosmic_session::CosmicSessionProxy;
use logind_zbus::{
    manager::ManagerProxy,
    session::{SessionClass, SessionProxy, SessionType},
    user::UserProxy,
};
use rustix::process::getuid;
use zbus::Connection;

#[derive(Debug, Clone, Copy)]
pub enum PowerAction {
    Lock,
    LogOut,
    Suspend,
    Restart,
    Shutdown,
}
impl PowerAction {
    pub fn perform(self) -> cosmic::iced::Task<cosmic::Action<app::Message>> {
        let msg = |m| cosmic::action::app(app::Message::Zbus(m));
        match self {
            PowerAction::Lock => cosmic::iced::Task::perform(lock(), msg),
            PowerAction::LogOut => cosmic::iced::Task::perform(log_out(), msg),
            PowerAction::Suspend => cosmic::iced::Task::perform(suspend(), msg),
            PowerAction::Restart => cosmic::iced::Task::perform(restart(), msg),
            PowerAction::Shutdown => cosmic::iced::Task::perform(shutdown(), msg),
        }
    }
}

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
