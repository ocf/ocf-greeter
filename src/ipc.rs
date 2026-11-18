use greetd_ipc::{AuthMessageType, ErrorType, Request, Response, codec::SyncCodec};
use std::{env, os::unix::net::UnixStream};

#[derive(Debug)]
pub enum LoginResult {
    Success,
    Failure,
}

// adapted from https://github.com/kennylevinsen/greetd/blob/master/agreety/src/main.rs
pub async fn login(
    cmd: &str,
    username: String,
    password: String,
) -> Result<LoginResult, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(env::var("GREETD_SOCK")?)?;
    let mut next_request = Request::CreateSession {
        username: username.clone(),
    };
    let mut starting = false;
    loop {
        next_request.write_to(&mut stream)?;

        match Response::read_from(&mut stream)? {
            Response::AuthMessage {
                auth_message,
                auth_message_type,
            } => {
                let response = match auth_message_type {
                    AuthMessageType::Visible => Some(username.clone()),
                    AuthMessageType::Secret => Some(password.clone()),
                    AuthMessageType::Info => {
                        eprintln!("info: {auth_message}");
                        None
                    }
                    AuthMessageType::Error => {
                        eprintln!("error: {auth_message}");
                        None
                    }
                };

                next_request = Request::PostAuthMessageResponse { response };
            }
            Response::Success => {
                if starting {
                    return Ok(LoginResult::Success);
                } else {
                    starting = true;
                    next_request = Request::StartSession {
                        env: vec![],
                        cmd: vec![cmd.to_string()],
                    }
                }
            }
            Response::Error {
                error_type,
                description,
            } => {
                Request::CancelSession.write_to(&mut stream)?;
                match error_type {
                    ErrorType::AuthError => return Ok(LoginResult::Failure),
                    ErrorType::Error => return Err(format!("login error: {description:?}").into()),
                }
            }
        }
    }
}
