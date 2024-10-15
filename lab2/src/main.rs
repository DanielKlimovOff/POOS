#[tokio::main]
async fn main() {
    let db = models::open_db("database.sqlitedb");

    let routes = filters::site(db);

    warp::serve(routes).run(([127, 0, 0, 1], 2017)).await;
}

mod models {
    use rusqlite::Connection;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use serde_derive::{Deserialize, Serialize};
    
    pub type Database = Arc<Mutex<Connection>>;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct User {
        id: i64,
        name: String,
        auth_hash: String,   
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Session {
        id: i64,
        hash: String,
        is_auth: bool,   
        user_id: Option<i32>,
        name: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Calculation {
        id: i64,
        num1: f64,
        num2: f64,
        operator_id: i64,
        result: f64,
        session_id: i64,
        user_id: Option<i64>,
    }

    pub fn open_db(name_db: &str) -> Database {
        let db = Connection::open(name_db).unwrap();
        let db = Arc::new(Mutex::new(db));
        db
    }
}

mod filters {
    use warp::{reply::Reply, Filter};
    use crate::models::Database;

    pub fn site(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        api(db)
            .or(data())
            .or(pages())
    }

    pub fn api(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("api")
            .map(|| {
                warp::reply()
            })
    }

    pub fn data() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("data")
            .map(|| {
                warp::reply()
            })
    }

    pub fn pages() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::any()
            .and_then(|| async {
                Ok(warp::reply().into_response())
            })
    }
}
