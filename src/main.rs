use iced::keyboard::{self, Key, key};
use iced::widget::{
    bottom, bottom_right, button, column, container, image, operation, pick_list, stack, text,
    text_input,
};
use iced::{
    ContentFit, Element, Event, Length, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
};
use std::env;
use std::path::Path;
use sys_info::linux_os_release;
mod ipc;
mod sessions;

#[derive(Debug, Clone)]
enum Message {
    UsernameUpdate(String),
    PasswordUpdate(String),
    Login,
    FocusNext,
    FocusPrevious,
    SessionSelected(sessions::DesktopSession),
}

#[derive(Default)]
struct State {
    username: String,
    password: String,
    sessions: Vec<sessions::DesktopSession>,
    selected_session: sessions::DesktopSession,
}

fn view(state: &State) -> Element<'_, Message> {
    stack![
        image("/etc/ocf-greeter/background.png")
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Cover),
        container(
            container(
                column![
                    text_input("Username", &state.username).on_input(Message::UsernameUpdate),
                    text_input("Password", &state.password)
                        .on_input(Message::PasswordUpdate)
                        .secure(true),
                    button("Login").on_press(Message::Login),
                ]
                .spacing(5),
            )
            .style(container::rounded_box)
            .padding(30)
            .width(400)
            .height(200)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill),
        bottom_right(pick_list(
            state.sessions.as_slice(),
            Some(&state.selected_session),
            Message::SessionSelected
        ))
        .padding(5),
        bottom(text(linux_os_release().unwrap().build_id.unwrap())).padding(5)
    ]
    .into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::UsernameUpdate(content) => {
            state.username = content;
            Task::none()
        }
        Message::PasswordUpdate(content) => {
            state.password = content;
            Task::none()
        }
        Message::Login => {
            println!("username: {}, password: {}", state.username, state.password);
            let result = ipc::login(
                &state.selected_session.exec,
                state.username.clone(),
                state.password.clone(),
            );
            println!("{:?}", result);
            Task::none()
        }
        Message::FocusNext => operation::focus_next(),
        Message::FocusPrevious => operation::focus_previous(),
        Message::SessionSelected(session) => {
            state.selected_session = session;
            Task::none()
        }
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _window| match event {
        Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
            Key::Named(key::Named::Tab) => {
                if modifiers.shift() {
                    Some(Message::FocusPrevious)
                } else {
                    Some(Message::FocusNext)
                }
            }
            Key::Named(key::Named::Enter) => Some(Message::Login),
            _ => None,
        },
        _ => None,
    })
}

pub fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    let sessions_path = Path::new(args.get(1).expect("expected a path to the sessions folder"));
    let sessions = sessions::get_desktop_sessions(sessions_path);
    let default_session_arg = args.get(2).expect("expected a default session");

    let default_session = sessions
        .iter()
        .find(|item| &item.name == default_session_arg)
        .cloned()
        .expect("failed default session matching");

    println!("{:?}", sessions);

    iced::application(
        move || State {
            sessions: sessions.clone(),
            selected_session: default_session.clone(),
            ..Default::default()
        },
        update,
        view,
    )
    .subscription(subscription)
    .theme(Theme::CatppuccinLatte)
    .run()
}
