//! Endpoints for filter rooms.
use bodyparser;
use iron::{Chain, Handler, IronError, IronResult, Plugin, Request, Response};
use iron::status::Status;
use serde_json::de::from_str;
use serde_json::value::ToJson;

use db::DB;
use error::ApiError;
use middleware::{AccessTokenAuth, FilterIdParam, JsonRequest, MiddlewareChain, UserIdParam};
use models::filter::{Filter, ContentFilter};
use models::user::User;
use modifier::SerializableResponse;

/// The GET `/user/:user_id/filter/:filter_id` endpoint.
pub struct GetFilter;

middleware_chain!(GetFilter, [AccessTokenAuth, FilterIdParam, UserIdParam]);

impl Handler for GetFilter {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let user_id = request.extensions.get::<UserIdParam>()
            .expect("UserIdParam should ensure a UserId").clone();

        let filter_id = request.extensions.get::<FilterIdParam>()
            .expect("FilterIdParam should ensure a FilterIdParam").clone();

        let connection = DB::from_request(request)?;
        let filter = Filter::find(&connection, user_id, filter_id)?;
        let response: ContentFilter = from_str(&filter.content).map_err(ApiError::from)?;
        Ok(Response::with((Status::Ok, SerializableResponse(response))))
    }
}

/// The POST `/user/:user_id/filter` endpoint.
pub struct PostFilter;

#[derive(Debug, Serialize)]
struct PostFilterResponse {
    filter_id: String,
}

middleware_chain!(PostFilter, [JsonRequest, AccessTokenAuth, UserIdParam]);

impl Handler for PostFilter {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        let user_id = request.extensions.get::<UserIdParam>()
            .expect("UserIdParam should ensure a UserId").clone();

        let user = request.extensions.get::<User>()
            .expect("AccessTokenAuth should ensure a user").clone();

        if user_id != user.id {
            Err(ApiError::unauthorized("The given user_id does not correspond to the authenticated user".to_string()))?;
        }

        let filter = match request.get::<bodyparser::Struct<ContentFilter>>() {
            Ok(Some(account_password_request)) => account_password_request,
            Ok(None) | Err(_) => {
                let error = ApiError::bad_json(None);
                return Err(IronError::new(error.clone(), error));
            }
        };

        let connection = DB::from_request(request)?;

        let id = Filter::create(&connection, user_id, filter. to_json().to_string())?;

        let response = PostFilterResponse {
            filter_id: id.to_string(),
        };

        Ok(Response::with((Status::Ok, SerializableResponse(response))))
    }
}

#[cfg(test)]
mod tests {
    use test::Test;
    use iron::status::Status;

    #[test]
    fn basic_test() {
        let test = Test::new();
        let access_token = test.create_access_token_with_username("carl");
        let user_id = "@carl:ruma.test";

        let filter_id = test.create_filter(&access_token, user_id, r#"{"room":{"timeline":{"limit":10}}}"#);

        let get_filter_path = format!(
            "/_matrix/client/r0/user/{}/filter/{}?access_token={}",
            user_id,
            filter_id,
            access_token
        );

        let response = test.get(&get_filter_path);
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.body, r#"{"room":{"timeline":{"limit":10}}}"#);
    }

    #[test]
    fn invalid_user() {
        let test = Test::new();
        let _ = test.create_access_token_with_username("carl");
        let alice = test.create_access_token_with_username("alice");
        let user_id = "@carl:ruma.test";
        let filter_path = format!(
            "/_matrix/client/r0/user/{}/filter?access_token={}",
            user_id,
            alice
        );

        let response = test.post(&filter_path, r#"{"room":{"timeline":{"limit":10}}}"#);
        assert_eq!(response.status, Status::Forbidden);
    }

    #[test]
    fn get_not_found() {
        let test = Test::new();
        let access_token = test.create_access_token_with_username("carl");
        let user_id = "@carl:ruma.test";

        let get_filter_path = format!(
            "/_matrix/client/r0/user/{}/filter/{}?access_token={}",
            user_id,
            1,
            access_token
        );

        let response = test.get(&get_filter_path);
        assert_eq!(response.status, Status::NotFound);
    }
}
