//! Endpoints for joining rooms.

use iron::{Chain, Handler, IronResult, Request, Response};
use iron::status::Status;

use db::DB;
use config::Config;
use middleware::{AccessTokenAuth, JsonRequest, RoomIdParam};
use modifier::SerializableResponse;
use room_membership::{RoomMembership, RoomMembershipOptions};
use user::User;

/// The `/rooms/:room_id/join` endpoint.
pub struct JoinRoom;

#[derive(Debug, Serialize)]
struct JoinRoomResponse {
    room_id: String,
}

impl JoinRoom {
    /// Create a `JoinRoom` with all necessary middleware.
    pub fn chain() -> Chain {
        let mut chain = Chain::new(JoinRoom);

        chain.link_before(JsonRequest);
        chain.link_before(RoomIdParam);
        chain.link_before(AccessTokenAuth);

        chain
    }
}

impl Handler for JoinRoom {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let user = request.extensions
            .get::<User>()
            .expect("AccessTokenAuth should ensure a user")
            .clone();

        let connection = DB::from_request(request)?;
        let config = Config::from_request(request)?;


        let room_id = request.extensions.get::<RoomIdParam>()
            .expect("Should have been required by RoomIdParam.")
            .clone();

        let room_membership_options = RoomMembershipOptions {
            room_id: room_id.clone(),
            user_id: user.id.clone(),
            sender: user.id,
            membership: String::from("join"),
        };
        let room_membership = RoomMembership::create(&connection, &config.domain, room_membership_options)?;

        let response = JoinRoomResponse { room_id: room_membership.room_id.to_string() };

        Ok(Response::with((Status::Ok, SerializableResponse(response))))
    }
}

#[cfg(test)]
mod tests {
    use test::Test;
    use iron::status::Status;

    #[test]
    fn join_room() {
        let test = Test::new();
        let access_token = test.create_access_token();
        let room_id = test.create_public_room(&access_token);

        let room_join_path = format!(
            "/_matrix/client/r0/rooms/{}/join?access_token={}",
            room_id,
            access_token
        );
        let response = test.post(&room_join_path, r"{}");

        assert_eq!(response.status, Status::Ok);
        assert!(response.json().find("room_id").unwrap().as_str().is_some());
    }

    #[test]
    fn rate_limited() {
        let test = Test::new();
        let access_token = test.create_access_token();
        let room_id = test.create_public_room(&access_token);

        let room_join_path = format!(
            "/_matrix/client/r0/rooms/{}/join?access_token={}",
            room_id,
            access_token
        );
        test.post(&room_join_path, r"{}");

        let room_join_path = format!(
            "/_matrix/client/r0/rooms/{}/join?access_token={}",
            room_id,
            access_token
        );
        let response = test.post(&room_join_path, r"{}");

        assert_eq!(response.status, Status::TooManyRequests);
        assert_eq!(
            response.json().find("errcode").unwrap().as_str().unwrap(),
            "M_LIMIT_EXCEEDED"
        );
        assert_eq!(
            response.json().find("error").unwrap().as_str().unwrap(),
            "Try to set the membership again!"
        );
    }

    #[test]
    fn forbidden_joining_private_room() {
        let test = Test::new();
        let access_token = test.create_access_token();
        let room_id = test.create_private_room(&access_token);

        let room_join_path = format!(
            "/_matrix/client/r0/rooms/{}/join?access_token={}",
            room_id,
            access_token
        );
        let response = test.post(&room_join_path, r"{}");

        assert_eq!(response.status, Status::Forbidden);
        assert_eq!(
            response.json().find("errcode").unwrap().as_str().unwrap(),
            "M_FORBIDDEN"
        );
        assert_eq!(
            response.json().find("error").unwrap().as_str().unwrap(),
            "You are not invited to this room."
        );
    }
}
