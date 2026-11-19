use iced::keyboard::{self, Key, key};
use iced::widget::{
    Id, bottom, bottom_right, button, column, container, image, operation, pick_list, stack, text,
    text_input,
};
use iced::{
    ContentFit, Element, Event, Length, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    color,
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
    LoginFailure,
    LoginError(String),
    LoginSuccess,
    FocusNext,
    FocusPrevious,
    SessionSelected(sessions::DesktopSession),
}

#[derive(Default, Clone)]
struct State {
    username: String,
    password: String,
    sessions: Vec<sessions::DesktopSession>,
    selected_session: sessions::DesktopSession,
    error_text: String,
    login_in_progress: bool,
}

const USERNAME_FIELD_ID: Id = Id::new("username_field_id");

async fn login_request(state: State) -> Message {
    let response = ipc::login(
        &state.selected_session.exec,
        state.username.clone(),
        state.password.clone(),
    )
    .await;
    println!("{:?}", response);

    match response {
        Ok(result) => match result {
            ipc::LoginResult::Success => Message::LoginSuccess,
            ipc::LoginResult::Failure => Message::LoginFailure,
        },
        Err(e) => {
            println!("ERROR: {}", e);
            Message::LoginError(e.to_string())
        }
    }
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
                    text_input("Username", &state.username)
                        .id(USERNAME_FIELD_ID)
                        .on_input(Message::UsernameUpdate)
                        .padding(10),
                    text_input("Password", &state.password)
                        .on_input(Message::PasswordUpdate)
                        .secure(true)
                        .padding(10),
                    button(
                        text("Login")
                            .width(Length::Fill)
                            .align_x(Horizontal::Center),
                    )
                    .on_press_maybe((!(state.login_in_progress)).then_some(Message::Login))
                    .width(Length::Fill)
                    .padding(10),
                    text(&state.error_text).color(color!(255, 0, 0)),
                ]
                .spacing(5),
            )
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
            match state.login_in_progress || state.username.is_empty() || state.password.is_empty()
            {
                false => {
                    state.login_in_progress = true;
                    Task::future(login_request(state.clone()))
                }
                true => Task::none(),
            }
        }
        Message::FocusNext => operation::focus_next(),
        Message::FocusPrevious => operation::focus_previous(),
        Message::SessionSelected(session) => {
            state.selected_session = session;
            Task::none()
        }
        Message::LoginSuccess => {
            state.login_in_progress = false;
            Task::none()
        }
        Message::LoginFailure => {
            state.login_in_progress = false;
            state.error_text = "Login Failed".into();
            Task::none()
        }
        Message::LoginError(error) => {
            state.login_in_progress = false;
            state.error_text = format!("Error: {}", error);
            println!("{:?}", error);
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

fn boot() -> (State, Task<Message>) {
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

    (
        State {
            sessions: sessions.clone(),
            selected_session: default_session.clone(),
            ..Default::default()
        },
        operation::focus(USERNAME_FIELD_ID),
    )
}

pub fn main() -> iced::Result {
    iced::application(boot, update, view)
        .title("OCF Greeter")
        .subscription(subscription)
        .theme(Theme::CatppuccinLatte)
        .run()
}
