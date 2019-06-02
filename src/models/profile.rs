//! Matrix profile.

use diesel::dsl::any;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use ruma_identifiers::UserId;

use crate::error::ApiError;
use crate::models::presence_status::PresenceStatus;
use crate::models::room_membership::{RoomMembership, RoomMembershipOptions};
use crate::schema::profiles;

/// A Matrix profile.
#[derive(AsChangeset, Debug, Clone, Identifiable, Insertable, Queryable)]
#[table_name = "profiles"]
pub struct Profile {
    /// The user's ID.
    pub id: UserId,
    /// The avatar url.
    pub avatar_url: Option<String>,
    /// The display name.
    pub displayname: Option<String>,
}

impl Profile {
    /// Update or Create a `Profile` entry with new avatar_url.
    pub fn update_avatar_url(
        connection: &PgConnection,
        homeserver_domain: &str,
        user_id: UserId,
        avatar_url: Option<String>,
    ) -> Result<Self, ApiError> {
        connection
            .transaction::<Self, ApiError, _>(|| {
                let maybe_profile = Self::find_by_uid(connection, &user_id)?;

                let profile = if let Some(mut profile) = maybe_profile {
                    profile.set_avatar_url(connection, avatar_url)?
                } else {
                    let new_profile = Self {
                        id: user_id.clone(),
                        avatar_url,
                        displayname: None,
                    };

                    Self::create(connection, &new_profile)?
                };

                PresenceStatus::upsert(connection, homeserver_domain, &user_id, None, None)?;

                Ok(profile)
            })
            .map_err(ApiError::from)
    }

    /// Update or Create a `Profile` entry with new displayname.
    pub fn update_displayname(
        connection: &PgConnection,
        homeserver_domain: &str,
        user_id: UserId,
        displayname: Option<String>,
    ) -> Result<Self, ApiError> {
        connection
            .transaction::<Self, ApiError, _>(|| {
                let maybe_profile = Self::find_by_uid(connection, &user_id)?;

                let profile = if let Some(mut profile) = maybe_profile {
                    profile.set_displayname(connection, displayname)?
                } else {
                    let new_profile = Self {
                        id: user_id.clone(),
                        avatar_url: None,
                        displayname,
                    };

                    Self::create(connection, &new_profile)?
                };

                PresenceStatus::upsert(connection, homeserver_domain, &user_id, None, None)?;

                Ok(profile)
            })
            .map_err(ApiError::from)
    }

    /// Update a `Profile` entry with new avatar_url.
    fn set_avatar_url(
        &mut self,
        connection: &PgConnection,
        avatar_url: Option<String>,
    ) -> Result<Self, ApiError> {
        self.avatar_url = avatar_url;

        match self.save_changes::<Self>(connection) {
            Ok(_) => Ok(self.clone()),
            Err(error) => Err(ApiError::from(error)),
        }
    }

    /// Update a `Profile` entry with new displayname.
    fn set_displayname(
        &mut self,
        connection: &PgConnection,
        displayname: Option<String>,
    ) -> Result<Self, ApiError> {
        self.displayname = displayname;

        match self.save_changes::<Self>(connection) {
            Ok(_) => Ok(self.clone()),
            Err(error) => Err(ApiError::from(error)),
        }
    }

    /// Update `RoomMembership`'s due to changed `Profile`.
    pub fn update_memberships(
        connection: &PgConnection,
        homeserver_domain: &str,
        user_id: UserId,
    ) -> Result<(), ApiError> {
        let mut room_memberships = RoomMembership::find_by_uid(connection, user_id.clone())?;

        for room_membership in &mut room_memberships {
            let options = RoomMembershipOptions {
                room_id: room_membership.room_id.clone(),
                user_id: user_id.clone(),
                sender: user_id.clone(),
                membership: "join".to_string(),
            };

            room_membership.update(connection, homeserver_domain, options)?;
        }

        Ok(())
    }

    /// Create a `Profile` entry.
    pub fn create(connection: &PgConnection, new_profile: &Self) -> Result<Self, ApiError> {
        diesel::insert_into(profiles::table)
            .values(new_profile)
            .get_result(connection)
            .map_err(ApiError::from)
    }

    /// Return `Profile` for given `UserId`.
    pub fn find_by_uid(
        connection: &PgConnection,
        user_id: &UserId,
    ) -> Result<Option<Self>, ApiError> {
        let profile = profiles::table.find(user_id).get_result(connection);

        match profile {
            Ok(profile) => Ok(Some(profile)),
            Err(DieselError::NotFound) => Ok(None),
            Err(err) => Err(ApiError::from(err)),
        }
    }

    /// Return `Profile`s for a list of `UserId`'s.
    pub fn get_profiles(
        connection: &PgConnection,
        users: &[UserId],
    ) -> Result<Vec<Self>, ApiError> {
        profiles::table
            .filter(profiles::id.eq(any(users)))
            .get_results(connection)
            .map_err(ApiError::from)
    }
}
