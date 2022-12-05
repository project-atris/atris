use std::rc::Rc;
use std::sync::Arc;

use atris_client_lib::atris_common::create_room::{CreateRoomResponse, CreateRoomError};
use atris_client_lib::atris_common::join_room::JoinRoomResponse;
use atris_client_lib::atris_common::{CipherKey};
use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::comms::AtrisChannel;
use atris_client_lib::comms::responder::AtrisResponder;
use client::{AtrisClient};
use iced::executor;
use iced::widget::{button, container, text,text_input, Column, radio};
use iced::{
    Alignment, Application, Command, Element, Length, Settings,
    Theme,
};

mod client;

#[derive(Debug,Clone, Copy,PartialEq, Eq)]
pub enum LoginMode {
    CreateUser,
    LoginUser
}

pub struct Session(CipherKey);

pub enum Atris {
    CreatingClient,
    ErrorCreatingClient,
    Login {
        atris_client: Arc<AtrisClient>,
        username:String,
        password:String,
        login_select: LoginMode,
        error_message: Option<String>,
        // Maybe this should also be create?
    },
    LoggingIn,
    Home {
        atris_client: Arc<AtrisClient>,
        session:Session,
        other_user:String,
        room_id:String,
    },
    CreateRoomError {
        other_user:String
    },
    MesageWaitingFailed(String),
    MessageWaitingPage {
        room_id:u16,
        other_user:Option<String>
    },
    MessagePage {
        room_id:u16,
        messages: Vec<String>,
        message_channel: AtrisChannel<String>
    }
    // 
}
#[derive(Debug,Clone)]
pub enum Message {
    // LoginPage
    UpdateUsername(String),
    UpdatePassword(String),
    ClientCreated(Result<Arc<AtrisClient>,()>),
    LoginComplete(Result<AuthenticateUserResponse,String>,Arc<AtrisClient>,LoginMode),
    LoginSelector(LoginMode),
    
    UpdateOtherUser(String),
    UpdateRoomId(String),

    RoomWaitingFailed(String),

    CreateRoom,
    CreateRoomFinished((Result<CreateRoomResponse, client::ClientError>,String)),
    JoinRoom,
    JoinRoomFinished(Result<JoinRoomResponse, client::ClientError>),

    MessageChannelRecieved(Arc<AtrisChannel<String>>),

    // RoomCreated(Result<AuthenticateUserResponse,String>,Arc<AtrisClient>),
    SubmitUserInfo,
    CreateClient,
}

impl Application for Atris {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self::CreatingClient,
            Command::perform(async {Message::CreateClient},|a|a)
        )
    }

    fn title(&self) -> String {
        "Example".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if let Message::CreateClient = message {
            return Command::perform(async {
                (AtrisClient::new().await).map(Arc::new).map_err(|_|{})
            },Message::ClientCreated);
        };
        match self {
            Self::Home { atris_client, session, other_user, room_id } =>{
                match message {
                    Message::UpdateOtherUser(u)=>{
                        *other_user = u;
                        Command::none()
                    },
                    Message::UpdateRoomId(r)=>{
                        *room_id = r;
                        Command::none()
                    },
                    Message::CreateRoom => {
                        let atris_client = Arc::clone(atris_client);
                        let other_user = other_user.clone();
                        let cipher_key = session.0.clone();
                        Command::perform( async move {
                            (atris_client.create_room(cipher_key, &other_user).await,other_user)
                        }, Message::CreateRoomFinished)
                    },
                    Message::CreateRoomFinished((r,other_user)) => {
                        match r {
                            Ok(room)=>{
                                let Self::Home { atris_client, session, other_user, room_id }  = std::mem::replace(self,Self::MessageWaitingPage {room_id:room.room_id,other_user:Some(other_user.clone()) }) else {
                                    unreachable!()
                                };
                                let atris_client =Arc::new(atris_client);
                                Command::perform(async move {
                                    let atris_responder = match AtrisResponder::new().await {
                                        Ok(responder)=>{
                                            match responder
                                            .into_channel_parts_with::<String>(&room.initiator_string)
                                            .await { 
                                                Ok((responder_string, channel_future))=>{
                                                    Ok((atris_client
                                                        .set_room_responder(
                                                            room.room_id,
                                                            session.0,
                                                            other_user.clone(),
                                                            responder_string
                                                        ).await,channel_future.await))
                                                },
                                                Err(e)=>Err(e),
                                            }
                                        }
                                        Err(e)=>Err(e)
                                    };
                                    if let Ok((Ok(set_room_response),Some(channel))) = atris_responder {
                                        let channel = AtrisChannel::new(channel, set_room_response.room_symmetric_key.as_cipher());
                                        Message::MessageChannelRecieved(Arc::new(channel))
                                    }else{
                                        Message::RoomWaitingFailed("A".into())
                                    }
                                }, |a|a)
                            }
                            Err(e) => {
                                *self = Self::CreateRoomError { other_user };
                                Command::none()
                            },
                        }
                    },
                    
                    Message::JoinRoom => {
                        let room_id = room_id.parse::<u16>();
                        let session = session.0.clone();
                        let atris_client = Arc::clone(atris_client);
                        if let Ok(room_id)=room_id {
                            Command::perform(async move {
                                atris_client.join_room(session, room_id).await
                            }, Message::JoinRoomFinished)
                        } else {
                            Command::none()
                        }
                    },
                    _=>unreachable!()
                }
            }
            Self::Login { username, password, login_select,.. } => {
                match message {
                    Message::UpdateUsername(s) => {
                        *username = s;
                        Command::none()
                    },
                    Message::UpdatePassword(s) => {
                        *password = s;
                        Command::none()
                    },
                    Message::SubmitUserInfo => {
                        println!("hi, {}, whose password is {}", username, password);
                        let Self::Login { atris_client, username, password,login_select, .. } = std::mem::replace(self,Self::LoggingIn) else {
                            unreachable!()
                        };
                        Command::perform(async move {
                            let res = if login_select == LoginMode::CreateUser {
                                let create_user_response = atris_client.create_user(&username.clone(), &password.clone()).await.map_err(|e|{
                                    format!("{e:?}")
                                });
                                match create_user_response {
                                    Ok(_)=>{
                                        atris_client.login(&username.clone(), &password.clone()).await.map_err(|e|{
                                            format!("{e:?}")
                                        })
                                    }
                                    Err(s)=>Err(s)
                                }
                            } else {
                                atris_client.login(&username.clone(), &password.clone()).await.map_err(|e|{
                                    format!("{e:?}")
                                })
                            };
                            Message::LoginComplete(res,atris_client,login_select)
                        },|a|a)
                    },
                    Message::LoginSelector(b)=>{
                        *login_select = b;
                        Command::none()
                    }
                    _=>unreachable!(),
                }
            },
            Self::CreatingClient => {
                match message {
                    Message::ClientCreated(c)=>{
                        *self = match c {
                            Ok(atris_client)=>Self::Login { error_message:None, atris_client, username: "".into(), password: "".into(),login_select:LoginMode::CreateUser },
                            Err(_)=>Self::ErrorCreatingClient
                        }
                    },
                    _=>unreachable!(),
                }
                Command::none()
            },
            Self::ErrorCreatingClient { .. } =>{
                unreachable!()
            }
            Self::LoggingIn => {
                match message {
                    Message::LoginComplete(result,atris_client,login_select)=>{
                        dbg!("Login complete!");
                        match result {
                            Ok(session)=>{
                                *self = Self::Home { atris_client,session: Session(session.session_id),room_id:"".into(),other_user:"".into() }
                            },
                            Err(error_message)=>{
                                *self = Self::Login { atris_client, username: "".into(), password: "".into(), error_message:Some(error_message), login_select }
                            }
                        }
                        Command::none()
                    }
                    _=>unreachable!()
                }
            }
            Self::MessagePage { room_id, messages, message_channel } => {
                Command::none()
            }
            _=>unreachable!()
        }
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match &self {
            Self::Login { username, password, error_message, login_select,.. } => {
                let username_input:Element<_> = text_input("Username", username, Message::UpdateUsername).into();
                let password_input:Element<_> = text_input("Password", password, Message::UpdatePassword).into();
                
                let login_selector: Element<_> = radio("Create new account", LoginMode::CreateUser, Some(*login_select), Message::LoginSelector).into();
                let login_selector1: Element<_> = radio("Login", LoginMode::LoginUser, Some(*login_select), Message::LoginSelector).into();
                
                let submit_button = button("Submit").on_press(Message::SubmitUserInfo);
                
                let mut inputs = Column::with_children(vec![
                    username_input,
                    password_input
                ]);
                if let Some(error_message)=error_message{
                    inputs=inputs.push(text(error_message))
                }
                Column::new()
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .push(
                        container(inputs)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y()
                        .padding(20)
                    )
                    .push(login_selector)
                    .push(login_selector1)
                    .push(submit_button)
                    .into()
            },
            Self::CreatingClient => {
                Column::with_children(vec![
                    text("Atris").into(),
                    text("Loading client..").into()
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
            },
            Self::Home {atris_client,session,other_user,room_id } => {
                Column::with_children(vec![
                    text_input("Enter a username",&other_user,Message::UpdateOtherUser).into(),
                    button(text({
                        if other_user.is_empty() {
                            "Create a room".into()
                        }else {
                            format!("Create a room with {}",other_user)
                        }
                    })).on_press(Message::CreateRoom).into(),
                    text_input("Enter a room_id",&room_id,Message::UpdateRoomId).into(),
                    button("Join room").on_press(Message::JoinRoom).into(),
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
            },
            Self::ErrorCreatingClient {  } => {
                Column::with_children(vec![
                    text("Error setting up.").into(),
                    button("Try again").on_press(Message::CreateClient).into()
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
            },
            Self::LoggingIn => {
                Column::with_children(vec![
                    text("Logging you in..").into(),
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
            },

            Self::MessagePage { room_id, message_channel,messages } => {
                Column::with_children(vec![
                    text(format!("Room {}",room_id)).into(),
                    text("Messages").into(),
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
                // Column::with_children(vec![
                //     text("Logging you in..").into(),
                // ])
                //     .spacing(10)
                //     .padding(10)
                //     .align_items(Alignment::Center)
                //     .into()
            },
            Atris::CreateRoomError { other_user } => {
                    Column::with_children(vec![
                        text(format!("Could not connect to {other_user}")).into(),
                        // button("Try again?").on_press(Message::U).into()
                    ])
                        .spacing(10)
                        .padding(10)
                        .align_items(Alignment::Center)
                        .into()
            },
            Atris::MesageWaitingFailed(s) => {
                Column::with_children(vec![
                    text(s).into(),
                    // button("Try again?").on_press(Message::U).into()
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
        },
            Atris::MessageWaitingPage { room_id, other_user } => {
                Column::with_children(vec![
                    text(format!("Waiting to join {room_id} with {:?}",other_user)).into(),
                ])
                    .spacing(10)
                    .padding(10)
                    .align_items(Alignment::Center)
                    .into()
        },
        }
    }
}

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (500,500);
    Atris::run(settings)
}
/*

MVC

M = {
    value: String
}

V = {
    value => text box
};

C = {
    self.value = value
};

*/