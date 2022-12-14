use std::borrow::Borrow;

use atris_common::{
    cipher::{ChaCha20Poly1305, KeyInit},
    set_room_responder::{
        SetRoomResponderError, SetRoomResponderRequest, SetRoomResponderResponse,
    },
    CipherKey, Encrypted, RoomData,
};
use atris_server::{
    room_table::AtrisRoomDBClient, run_lambda_http, session_table::AtrisSessionDBClient,
};

run_lambda_http!(
    |request: Request<SetRoomResponderRequest>| -> Result<SetRoomResponderResponse, SetRoomResponderError> {
        let (_, request) = request.into_parts();

        let session_table = AtrisSessionDBClient::new().await;
        let requester_session = session_table.get_session(request.session_id.clone()).await.ok().and_then(|a|a);
        let other_session = session_table.get_session_for_username(request.other_user_name.clone()).await.ok().and_then(|a|a);
        let requester_session = requester_session.ok_or(SetRoomResponderError::InvalidSessionId(request.session_id.clone()))?;
        let other_session = other_session.ok_or(SetRoomResponderError::NoSessionForUser(request.other_user_name))?;

        let room_table = AtrisRoomDBClient::new().await;

        let borrowed_key = other_session.session_id.borrow();
        dbg!(&borrowed_key);
        let mut cipher = ChaCha20Poly1305::new(borrowed_key);

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
