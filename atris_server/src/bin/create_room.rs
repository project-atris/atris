use std::borrow::Borrow;

use argon2::{Argon2, PasswordHasher};
use atris_common::{create_user::*, create_room::{CreateRoomRequest, CreateRoomResponse, CreateRoomError}, Cipher, CipherKey, RoomData, Encrypted, cipher::{ChaCha20Poly1305, KeyInit}};
use atris_server::{room_table::{AtrisRoomDBClient, Room}, run_lambda_http, session_table::AtrisSessionDBClient};

use password_hash::SaltString;

run_lambda_http!(
    |request: Request<CreateRoomRequest>| -> Result<CreateRoomResponse, CreateRoomError> {
        let (_, request) = request.into_parts();

        let room_table = AtrisRoomDBClient::new().await;
        let session_table = AtrisSessionDBClient::new().await;
        let requester_session = session_table.get_session(request.session_id.clone()).await.ok().and_then(|a|a);
        let other_session = session_table.get_session_for_username(request.other_user_name.clone()).await.ok().and_then(|a|a);
        
        let requester_session =requester_session.ok_or(CreateRoomError::InvalidSessionId(request.session_id.clone()))?;
        let other_session = other_session.ok_or(CreateRoomError::NoSessionForUser(request.other_user_name))?;
        let room_id = loop{
            let potential_id:u16 = rand::random();
            let Err(CreateRoomError::DuplicateRoomId(_)) = room_table.create_room(potential_id, requester_session.username.clone()).await else {
                break potential_id;
            };
        };
        dbg!(room_id);
        Ok(CreateRoomResponse {
            room_id,
            initiator_string:other_session.initiator
        })
    }
);
