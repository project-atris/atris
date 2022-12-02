use atris_common::join_room::*;

use atris_server::{
    room_table::AtrisRoomDBClient, run_lambda_http, session_table::AtrisSessionDBClient,
};

run_lambda_http!(
    |request: Request<JoinRoomRequest>| -> Result<JoinRoomResponse, JoinRoomError> {
        let (_, request) = request.into_parts();

        // Retrieve user from database
        let session_table = AtrisSessionDBClient::new().await;
        let requester_session = session_table
            .get_session(request.session_id.clone())
            .await
            .ok()
            .and_then(|a| a);

        let _requester_session =
            requester_session.ok_or(JoinRoomError::InvalidSessionId(request.session_id.clone()))?;

        let room_table = AtrisRoomDBClient::new().await;
        let room = room_table.get_room(request.room_id).await?;
        Ok(JoinRoomResponse {
            room_data: room.room_data,
        })
    }
);
