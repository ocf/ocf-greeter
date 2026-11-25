use iced::keyboard::{self, Key, key};
use iced::widget::{
    Id, Space, bottom, bottom_right, button, column, container, container::rounded_box, image,
    operation, pick_list, stack, text, text_input,
};
use iced::{
    Color, ContentFit, Element, Event, Font, Length, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    color, font, padding, window,
};
use std::path::Path;
use std::{env, fs};
use sys_info::linux_os_release;
mod ipc;
mod sessions;
use chrono::{DateTime, Local};
use iced_aw::Spinner;

#[derive(Default, Clone)]
enum LoginStatus {
    #[default]
    Idle,
    Loading,
    Complete,
}

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
    CheckCapsLock,
    ToggleOutOfOrder,
    Tick(DateTime<Local>),
}

#[derive(Default, Clone)]
struct State {
    username: String,
    password: String,
    sessions: Vec<sessions::DesktopSession>,
    selected_session: sessions::DesktopSession,
    error_text: String,
    login_status: LoginStatus,
    caps_lock_enabled: bool,
    out_of_order: bool,
    time: DateTime<Local>,
}

const USERNAME_FIELD_ID: Id = Id::new("username_field_id");

const ASSETS_ROOT: &str = if !cfg!(debug_assertions) {
    "/etc/ocf-greeter"
} else {
    "assets"
};

const WINDOW_DECORATIONS_ENABLED: bool = cfg!(debug_assertions);

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

fn is_caps_lock_on() -> bool {
    let Ok(entries) = fs::read_dir("/sys/class/leds") else {
        return false;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };

        if name_str.ends_with("::capslock") {
            let brightness_path = entry.path().join("brightness");
            if let Ok(val) = fs::read_to_string(&brightness_path) {
                if val.trim() == "1" {
                    return true;
                }
            }
        }
    }

    false
}

fn view(state: &State) -> Element<'_, Message> {
    let view: Element<'_, Message> = stack![
        image(if !state.out_of_order {
            format!("{}/background.png", ASSETS_ROOT)
        } else {
            // FIXME: add final out of order wallpaper
            format!("{}/out-of-order.png", ASSETS_ROOT)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(ContentFit::Cover),
        if state.out_of_order {
            // don't show login boxes if marked out of order
            container(Space::new())
        } else if matches!(state.login_status, LoginStatus::Complete) {
            // show spinner and hide login boxes while the DE is starting
            container(
                container(Spinner::new())
                    .style(rounded_box)
                    .padding(10)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            container(
                container(
                    column![
                        text_input("Username", &state.username)
                            .id(USERNAME_FIELD_ID)
                            .on_input(Message::UsernameUpdate)
                            .padding(10),
                        column![
                            text_input("Password", &state.password)
                                .on_input(Message::PasswordUpdate)
                                .secure(true)
                                .padding(10),
                            text(if state.caps_lock_enabled {
                                "Caps Lock is Enabled"
                            } else {
                                ""
                            }),
                        ]
                        .spacing(0.5),
                        button(
                            text("Login")
                                .width(Length::Fill)
                                .align_x(Horizontal::Center),
                        )
                        .on_press_maybe(
                            matches!(state.login_status, LoginStatus::Idle)
                                .then_some(Message::Login)
                        )
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
            .center_y(Length::Fill)
        },
        bottom_right(pick_list(
            state.sessions.as_slice(),
            Some(&state.selected_session),
            Message::SessionSelected,
        ))
        .padding(5),
        container(
            column![
                text(state.time.format("%H:%M").to_string())
                    .size(75)
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::default()
                    }),
                text(state.time.format("%Y-%m-%d").to_string()).size(25)
            ]
            .spacing(0)
            .align_x(Horizontal::Center)
        )
        .padding(padding::right(20.0))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Vertical::Top)
        .align_x(Horizontal::Right),
        bottom(text(linux_os_release().unwrap().build_id.unwrap())).padding(5)
    ]
    .into();

    if cfg!(feature = "debug_layout") {
        return view.explain(Color::BLACK);
    }
    view
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
            match !matches!(state.login_status, LoginStatus::Idle)
                || state.username.is_empty()
                || state.password.is_empty()
            {
                false => {
                    state.login_status = LoginStatus::Loading;
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
            state.login_status = LoginStatus::Complete;
            Task::none()
        }
        Message::LoginFailure => {
            state.login_status = LoginStatus::Idle;
            state.error_text = "Login Failed".into();
            Task::none()
        }
        Message::LoginError(error) => {
            state.login_status = LoginStatus::Idle;
            state.error_text = format!("Error: {}", error);
            println!("{:?}", error);
            Task::none()
        }
        Message::CheckCapsLock => {
            state.caps_lock_enabled = is_caps_lock_on();
            Task::none()
        }
        Message::ToggleOutOfOrder => {
            state.out_of_order = !state.out_of_order;
            Task::none()
        }
        Message::Tick(now) => {
            state.time = now;
            Task::none()
        }
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch(vec![
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
                Key::Character(c)
                    if (c == "o" || c == "O") && modifiers.shift() && modifiers.control() =>
                {
                    Some(Message::ToggleOutOfOrder)
                }
                _ => None,
            },
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => match key {
                Key::Named(key::Named::CapsLock) => Some(Message::CheckCapsLock),
                _ => None,
            },
            Event::Window(window::Event::Focused) => Some(Message::CheckCapsLock),
            _ => None,
        }),
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick(Local::now())),
    ])
}

fn boot() -> (State, Task<Message>) {
    let args: Vec<String> = env::args().collect();
    let sessions_path = Path::new(args.get(1).expect("expected a path to the sessions folder"));
    let sessions = sessions::get_desktop_sessions(sessions_path);
    let default_session_arg = args.get(2).expect("expected a default session");

    let default_session = sessions
        .iter()
        .find(|item| &item.id == default_session_arg)
        .cloned()
        .expect("failed default session matching");

    let caps_lock_state = is_caps_lock_on();

    let current_time = Local::now();

    println!("{:?}", sessions);

    (
        State {
            sessions: sessions.clone(),
            selected_session: default_session.clone(),
            caps_lock_enabled: caps_lock_state,
            time: current_time,
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
        .window(window::Settings {
            decorations: WINDOW_DECORATIONS_ENABLED,
            ..Default::default()
        })
        .run()
}
