use iced::widget::{button, column, container, text_input};
use iced::{
    Element, Length,
    alignment::{Horizontal, Vertical},
};

#[derive(Debug, Clone)]
enum Message {
    UsernameUpdate(String),
    PasswordUpdate(String),
    Login,
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

fn update(state: &mut State, message: Message) {
    match message {
        Message::UsernameUpdate(content) => state.username = content,
        Message::PasswordUpdate(content) => state.password = content,
        Message::Login => println!("username: {}, password: {}", state.username, state.password),
    }
}

pub fn main() -> iced::Result {
    iced::run(update, view)
}
