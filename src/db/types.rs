use diesel::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;
