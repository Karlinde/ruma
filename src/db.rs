//! Database-related functionality.

use diesel::pg::PgConnection;
use diesel::r2d2::{Builder, ConnectionManager, Pool, PoolError as R2d2Error, PooledConnection};
use iron::typemap::Key;
use iron::{Plugin, Request};
use persistent::Write;

use crate::error::ApiError;

/// An Iron plugin for attaching a database connection pool to an Iron request.
#[derive(Clone, Copy, Debug)]
pub struct DB;

impl DB {
    /// Creates a connection pool for the PostgreSQL database at the given URL.
    pub fn create_connection_pool(
        r2d2_pool_builder: Builder<ConnectionManager<PgConnection>>,
        postgres_url: &str,
    ) -> Result<Pool<ConnectionManager<PgConnection>>, R2d2Error> {
        let connection_manager = ConnectionManager::new(postgres_url);

        r2d2_pool_builder.build(connection_manager)
    }

    /// Extract a database conection from the pool stored in the request.
    pub fn from_request(
        request: &mut Request<'_, '_>,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, ApiError> {
        let mutex = request.get::<Write<Self>>().map_err(ApiError::from)?;
        let pool = mutex.lock().map_err(ApiError::from)?;
        pool.get().map_err(ApiError::from)
    }
}

impl Key for DB {
    type Value = Pool<ConnectionManager<PgConnection>>;
}
