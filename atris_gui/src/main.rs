use std::borrow::Borrow;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::sync::{Arc};
use std::vec;

use atris_client_lib::atris_common::create_room::{CreateRoomResponse, CreateRoomError};
use atris_client_lib::atris_common::join_room::JoinRoomResponse;
use atris_client_lib::atris_common::{CipherKey};
use atris_client_lib::atris_common::authenticate_user::AuthenticateUserResponse;
use atris_client_lib::comms::AtrisChannel;
use atris_client_lib::comms::responder::AtrisResponder;
use client::{AtrisClient};
use iced::alignment::Horizontal;
use iced::{executor, Subscription, subscription};
use iced::futures::lock::Mutex;
use iced::widget::{button, container, text,text_input, Column, radio, Row};
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
        messages: Vec<AtrisMessage>,
        current_message:String,
        message_channel: Arc<Mutex<AtrisChannel<AtrisMessageData>>>
    }
    // 
}
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum AtrisMessage {
    Sent(String),
    Received(String)
}

#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub enum AtrisMessageData {
    Text(String),
    File{
        #[serde(with = "serde_bytes")]
        data:Vec<u8>,
        name:String,
    }
}
// pub type AtrisMessageData = String;

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
    JoinRoomFinished(u16,Result<JoinRoomResponse, client::ClientError>),

    MessageChannelReceived(Arc<Mutex<AtrisChannel<AtrisMessageData>>>),
    ReceiveMessage(AtrisMessageData),
    ReceiveMessageFailed,

    SendMessage,
    MessageSent(String),

    UpdateCurrentMessage(String),
    SendFile, //includes the local directory of the file to send
    ActualSendFile(PathBuf),

    // RoomCreated(Result<AuthenticateUserResponse,String>,Arc<AtrisClient>),
    SubmitUserInfo,
    CreateClient,

    Nop
}

async fn wait_for_next_message_actually(channel:&Arc<Mutex<AtrisChannel<AtrisMessageData>>>)->Message {
    dbg!("Waiting for lock to receive");
    let mut lock = channel.lock().await;
    dbg!("Got lock to receive");
    let msg = lock.receive().await.and_then(|r|dbg!(r).ok()).map(Message::ReceiveMessage).unwrap_or(Message::ReceiveMessageFailed);
    drop(lock);
    msg
}
// fn wait_for_next_message(channel:Arc<Mutex<AtrisChannel<AtrisMessageData>>>) -> Command<Message> {
//     Command::perform(wait_for_next_message_actually(channel), |a|a)
// }

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

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        if let Self::MessagePage { message_channel,.. } = self {
            subscription::unfold((), message_channel.clone(), |channel|async move {
                let msg = {
                    let mut lock = channel.lock().await;
                    lock.try_receive().ok().map(Message::ReceiveMessage)
                };
                (msg,channel)
            })
        }else{
            Subscription::none()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if let Message::CreateClient = message {
            return Command::perform(async {
                (AtrisClient::new().await).map(Arc::new).map_err(|_|{})
            },Message::ClientCreated);
        }else if let Message::Nop = message {
            return Command::none()
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
                                            .into_channel_parts_with::<AtrisMessageData>(&room.initiator_string)
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
                                        Message::MessageChannelReceived(Arc::new(Mutex::new(channel)))
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
                            },move |r|Message::JoinRoomFinished(room_id,r))
                        } else {
                            Command::none()
                        }
                    },
                    Message::JoinRoomFinished(room_id, r) => {
                        if let Ok(join_room_response)=r {
                            println!("Decrypting");
                            let room_data = join_room_response.room_data.decrypt(&mut session.0.as_cipher()).unwrap();
                            println!("Done decrupting, swapping");
                            let Self::Home { atris_client, session, other_user, room_id } = std::mem::replace(self,Self::MessageWaitingPage {room_id,other_user:None }) else {
                                unreachable!()
                            };
                            println!("Done swapping, unwrapping");
                            if let Ok(c) = Arc::try_unwrap(atris_client) {
                                Command::perform(async move {
                                    println!("Done unwrapping, Making parts");
                                    let parts = c.initiator.into_channel_parts_with::<AtrisMessageData>(&room_data.responder_string).await.unwrap();
                                    println!("Making channel");
                                    let channel = AtrisChannel::new(parts, room_data.symmetric_key.as_cipher());
                                    println!("Done!");
                                    Message::MessageChannelReceived(Arc::new(Mutex::new(channel)))
                                },|a|a)
                            }else {
                                println!("Too many client havers!");
                                Command::none()
                            }
                        }else{
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
                            Ok(atris_client)=>Self::Login { error_message:None, atris_client, username: "".into(), password: "".into(),login_select:LoginMode::LoginUser },
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
            Self::MessageWaitingPage { room_id, other_user } => {
                match message {
                    Message::MessageChannelReceived(message_channel)=>{
                        *self = Self::MessagePage { room_id:*room_id, messages: Default::default(), current_message: Default::default(), message_channel };
                    }
                    Message::RoomWaitingFailed(msg)=>{
                        *self = Self::MesageWaitingFailed(msg);
                    }
                    _=>unreachable!()
                }
                Command::none()
            }
            Self::MessagePage { room_id, messages, current_message,message_channel,.. } => {
                match message {
                    Message::SendMessage => {
                        let message_channel = message_channel.clone();
                        let current_message = current_message.clone();
                        Command::perform(async move {
                            println!("Waiting for lock to send {current_message:?}");
                            let mut lock = message_channel.lock().await;
                            println!("Got lock to send {current_message:?}");                            
                            // lock.send(current_message.clone()).await;
                            lock.send(AtrisMessageData::Text(current_message.clone())).await;
                            drop(lock);
                            current_message
                        }, Message::MessageSent)
                    },
                    Message::UpdateCurrentMessage(m) => {
                        *current_message = m;
                        Command::none()
                    }
                    Message::ReceiveMessage(m)=>{
                        match m {
                            AtrisMessageData::Text(m)=>{
                                messages.push(AtrisMessage::Received(m));
                            }
                            AtrisMessageData::File { data, name }=>{
                                let full_path = format!("~/Downloads/{name}");
                                let mut path = Path::new(&full_path);
                                let res = if path.exists() {
                                    let ext = path.extension().unwrap_or_default();
                                    let prefix = path.with_extension("");
                                    let path = &(1..).find_map(|number|{
                                        let new_path = prefix.with_file_name(format!("{} ({number})",prefix.file_name().and_then(OsStr::to_str).unwrap_or(""))).with_extension(ext);
                                        if new_path.exists() {
                                            Some(new_path)
                                        }else {
                                            None
                                        }
                                    }).unwrap();
                                    std::fs::write(path, data)
                                }else {
                                    std::fs::write(path, data)
                                };

                                // res
                                
                                // file.txt
                                // file(1).txt
                                // file(2).txt
                            }
                            // m => {
                            //     messages.push(AtrisMessage::Received(Atrim));
                            // }
                            _=>unreachable!()
                        }
                        Command::none()
                        // wait_for_next_message(message_channel.clone())
                    }
                    Message::ReceiveMessageFailed=>{
                        Command::none()
                    }
                    Message::MessageSent(s)=>{
                        messages.push(AtrisMessage::Sent(s));
                        Command::none()
                    }
                    Message::ActualSendFile(path)=>{
                        let message_channel = message_channel.clone();
                        Command::perform(async move {
                            let bytes = match std::fs::read(&path) {
                                Ok(bytes)=>bytes,
                                _ => return Message::Nop
                            };
                            let filename = match path.as_path().file_name().and_then(OsStr::to_str) {
                                Some(f)=>f,
                                None=>return Message::Nop
                            };
                            let mut lock = message_channel.lock().await;
                            if lock.send(AtrisMessageData::File{
                                data:bytes,
                                name:filename.into()
                            }).await.is_err() {
                                return Message::Nop
                            };
                            Message::MessageSent(format!("File {filename}"))
                        }, |a|a)
                    }

                    Message::SendFile => {
                        Command::perform(async move {
                            let filepath = native_dialog::FileDialog::new().show_open_single_file();
                            let path = match filepath {
                                Ok(Some(path))=>path,
                                _ => return Message::Nop
                            };
                            Message::ActualSendFile(path)
                        }, |a|a)
                    }
                    d=>{
                        dbg!(d);
                        unreachable!()
                    }
                }
            }
            _=>unreachable!()
        }
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match &self {
            Self::Login { username, password, error_message, login_select,.. } => {
                let username_input:Element<_> = text_input("Username", username, Message::UpdateUsername).into();
                let password_input:Element<_> = text_input("Password", password, Message::UpdatePassword).into();
                
                let create_selector: Element<_> = radio("Create new account", LoginMode::CreateUser, Some(*login_select), Message::LoginSelector).into();
                let login_selector: Element<_> = radio("Login", LoginMode::LoginUser, Some(*login_select), Message::LoginSelector).into();

                let login_row:Element<_> = Row::with_children(vec![
                    login_selector,
                    create_selector
                ])
                .padding(20)
                .into();

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
                    .push(login_row)
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
            Self::Home {other_user,room_id,.. } => {
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

            Self::MessagePage { room_id,messages,current_message,.. } => {
                let mut header = vec![
                    text(format!("Room {}",room_id)).into(),
                    text("Messages: ").into(),
                ];

                header.extend(messages.iter().map(|m|{
                    match m {
                        AtrisMessage::Received(r)=>text(format!("Rec: {r}")).horizontal_alignment(Horizontal::Left),
                        AtrisMessage::Sent(s)=>text(format!("Sent: {s}")).horizontal_alignment(Horizontal::Right),
                    }.width(Length::Fill).into()
                }));

                let input = text_input("Enter a message", current_message, Message::UpdateCurrentMessage).into();
                let send = button("Send").on_press(Message::SendMessage).into();
                let send_file = button("Send File").on_press(Message::SendFile).into();

                let input_row: Row<_> = Row::with_children(vec![
                    send_file,
                    input,
                    send,
                ]).into();

                Column::with_children(header)
                    .spacing(10)
                    .padding(10)
                    .push(input_row)
                    // .align_items(Alignment::Center)
                    .into()
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
    settings.exit_on_close_request = true;
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