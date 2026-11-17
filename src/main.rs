use iced::keyboard::{self, Key, key};
use iced::widget::{button, column, container, operation, text_input};
use iced::{
    Element, Event, Length, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
};
mod ipc;

#[derive(Debug, Clone)]
enum Message {
    UsernameUpdate(String),
    PasswordUpdate(String),
    Login,
    FocusNext,
    FocusPrevious,
}

#[derive(Default)]
struct State {
    username: String,
    password: String,
}

fn view(state: &State) -> Element<'_, Message> {
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
    .center_y(Length::Fill)
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
            let result = ipc::login("id", state.username.clone(), state.password.clone());
            println!("{:?}", result);
            Task::none()
        }
        Message::FocusNext => operation::focus_next(),
        Message::FocusPrevious => operation::focus_previous(),
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
    iced::application(State::default, update, view)
        .subscription(subscription)
        .theme(Theme::CatppuccinLatte)
        .run()
}
