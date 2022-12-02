use std::borrow::Borrow;

use argon2::{Argon2, PasswordHasher};
use atris_common::{create_user::*, create_room::{CreateRoomRequest, CreateRoomResponse}, Cipher, CipherKey, RoomData, Encrypted, cipher::{ChaCha20Poly1305, KeyInit}, set_room_responder::{SetRoomResponderError, SetRoomResponderRequest, SetRoomResponderResponse}};
use atris_server::{room_table::{AtrisRoomDBClient, Room}, run_lambda_http, session_table::AtrisSessionDBClient};

use password_hash::SaltString;

run_lambda_http!(
    |request: Request<SetRoomResponderRequest>| -> Result<SetRoomResponderResponse, SetRoomResponderError> {
        let (_, request) = request.into_parts();

        let room_table = AtrisRoomDBClient::new().await;
        let session_table = AtrisSessionDBClient::new().await;
        let requester_session = session_table.get_session(request.session_id.clone()).await.ok().and_then(|a|a);
        let other_session = session_table.get_session_for_username(request.other_user_name.clone()).await.ok().and_then(|a|a);
        
        // dbg!(&requester_session.);

        let requester_session = requester_session.ok_or(SetRoomResponderError::InvalidSessionId(request.session_id.clone()))?;
        let other_session = other_session.ok_or(SetRoomResponderError::NoSessionForUser(request.other_user_name))?;

        let mut cipher = ChaCha20Poly1305::new(other_session.session_id.borrow());

        let room_symmetric_key = CipherKey::generate();

        let room_data = RoomData {
            responder_string: request.responder_string,
            symmetric_key: room_symmetric_key.clone()
        };
        let room_data = Encrypted::encrypt(&room_data, &mut cipher).map_err(|_|SetRoomResponderError::EncryptionError)?;
        room_table.update_room_data(request.room_id,requester_session.username,room_data).await?;

        Ok(SetRoomResponderResponse { room_symmetric_key })
    }
);
