//! Postgres service

use sqlx::postgres::PgPool;

/// Postgres interfaces
pub struct Postgres(PgPool);
