use std;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;

type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
// We use a global shared sqlite connection because it's simple and performance is not 
// very important
lazy_static! {
    pub static ref POOL: SqlitePool = pooled_sqlite();
}

fn pooled_sqlite() -> SqlitePool {
    let manager = ConnectionManager::<SqliteConnection>::new(std::env::var("DATABASE_URL").unwrap());
    Pool::new(manager).expect("Postgres connection pool could not be created")
}
