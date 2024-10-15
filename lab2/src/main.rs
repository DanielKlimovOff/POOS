use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    // if env::var_os("RUST_LOG").is_none() {
    //     // Set `RUST_LOG=site=debug` to see debug logs,
    //     // this only shows access logs.
    //     env::set_var("RUST_LOG", "site=info");
    // }
    // pretty_env_logger::init();
    
    let db = models::open_db("database.sqlitedb");

    let api = filters::site(db);
    let routes = api.with(warp::log("site"));

    warp::serve(routes).run(([127, 0, 0, 1], 2017)).await;
}

mod models {
    use rusqlite::Connection;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use serde_derive::{Deserialize, Serialize};
    
    pub type Database = Arc<Mutex<Connection>>;

    #[derive(Debug)]
    pub struct UnIdentified;
    impl warp::reject::Reject for UnIdentified {}

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct CalculateJson {
        pub num1: f64,
        pub num2: f64,
        pub operator_id: i32,
        pub result: Option<f64>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct User {
        id: i32,
        name: String,
        auth_hash: String,   
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Session {
        id: i32,
        hash: String,
        is_auth: bool,   
        user_id: Option<i32>,
        name: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Calculation {
        id: i32,
        num1: f64,
        num2: f64,
        operator_id: i32,
        result: f64,
        session_id: i32,
        user_id: Option<i32>,
    }

    pub fn open_db(name_db: &str) -> Database {
        let db = Connection::open(name_db).unwrap();
        let db = Arc::new(Mutex::new(db));
        db
    }
}

mod filters {
    use crate::{handlers, models};
    use warp::{reply::Reply, Filter};
    use crate::models::{Database, CalculateJson};

    pub fn site(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        check_cookies()
            // .recover(handlers::user_have_not_cookies_situation)
            .map(|_| ())
            .untuple_one()
            .and(
                api(db)
                .or(data())
                .or(pages())
            )
    }

    pub fn api(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("api").and(
            calculate(db.clone()))
                // .or(login(db.clone())) 
                // .or(register(db.clone()))
                // .or(history(db.clone()))
        // )
    }

    pub fn calculate(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("calculate")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::cookie("session_hash"))
            .and(json_body_calculate())
            .and(with_db(db))
            .and_then(handlers::calculate)
    }

    pub fn data() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("data")
            .and(warp::fs::dir("./data/"))
    }

    pub fn pages() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        home_page()
            .or(login_page())
            .or(history_page())
            .or(register_page())
    }

    pub fn home_page() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path::end()
            .and(warp::get())
            .and(warp::fs::file("./data/home/home.html"))
    }

    pub fn login_page() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("login")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::fs::file("./data/login/login.html"))
    }

    pub fn register_page() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("register")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::fs::file("./data/register/register.html"))
    }

    pub fn history_page() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("history")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::fs::file("./data/history/history.html"))
    }

    fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    fn json_body_calculate() -> impl Filter<Extract = (CalculateJson,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    fn check_cookies() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::any()
            .and(warp::cookie::optional("session_hash"))
            .and_then(|session_hash: Option<String>| async {
                if session_hash == None{
                    return Err(warp::reject::custom(models::UnIdentified));
                }
                Ok(session_hash.unwrap())
            })
    }

    // fn catch_header(db: Database) -> impl Filter<Extract = (_, ), Error = std::convert::Infallible> + Clone {
    //     warp::header("session_hash")
    // }
}

mod handlers {
    use crate::models::{CalculateJson, Database};
    use warp::reply::Reply;
    use warp::http::StatusCode;
    use rusqlite::params;
    use rand;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    pub async fn calculate(session_hash: String, input_data: CalculateJson, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        println!("hash123 - {}", session_hash.clone());
        // if session_hash == None{
        //     // let rand_int = rng.gen_range(0..114);
        //     let hash_seed = rand::random::<i32>();
        //     hash_seed.hash(&mut hasher);
        //     let new_session_hash = STANDARD.encode(hasher.finish().to_string());
        //     let new_session_name = "Udefiend ".to_string() + "Dazzle" + &hash_seed.to_string();
        //     let db_response = db.lock().await.execute("insert into sessions (hash, is_auth, name) values (?1, ?2, ?3);", params![&new_session_hash, false, &new_session_name]);
            
        //     match db_response {
        //         Ok(_) => return Ok(warp::reply::with_header(
        //             warp::reply(),
        //             "set-cookie",
        //             format!("session_hash={new_session_hash}")).into_response()),
        //         Err(massage) => {
        //             println!("{massage}");
        //             return Ok(warp::reply::with_status("ERROR_WITH_DB", StatusCode::INTERNAL_SERVER_ERROR).into_response());
        //         },
        //         // Error(error_massage) => Err(warp::reject::custom(error_on_db)),
        //     }
        // }

        let mut result_data = CalculateJson {
            num1: input_data.num1,
            num2: input_data.num2,
            operator_id: input_data.operator_id,
            result: None,
        };

        match input_data.operator_id {
            0 => result_data.result = Some(result_data.num1 + result_data.num2),
            1 => result_data.result = Some(result_data.num1 - result_data.num2),
            2 => result_data.result = Some(result_data.num1 * result_data.num2),
            3 => result_data.result = Some(result_data.num1 / result_data.num2),
            _ => result_data.result = None,
        }

        Ok(warp::reply::json(&result_data).into_response())
        // Ok(
        //     warp::reply::with_header(warp::reply::json(&result_data).into_response(),
        //     "set-cookie", "session_hash=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT")
        // )
    }

    pub async fn user_have_not_cookies_situation(db: Database, err: warp::Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
        let hash_seed = rand::random::<i32>();
        let mut hasher = DefaultHasher::new();
        hash_seed.hash(&mut hasher);
        let new_session_hash = STANDARD.encode(hasher.finish().to_string());
        let new_session_name = "Udefiend ".to_string() + "Dazzle" + &hash_seed.to_string();
        let db_response = db.lock().await.execute("insert into sessions (hash, is_auth, name) values (?1, ?2, ?3);", params![&new_session_hash, false, &new_session_name]);
        
        match db_response {
            Ok(_) => Ok(warp::reply::with_header(
                warp::reply(),
                "set-cookie",
                format!("session_hash={new_session_hash}")).into_response()),
            Err(massage) => {
                println!("{massage}");
                Ok(warp::reply::with_status("ERROR_WITH_DB", StatusCode::INTERNAL_SERVER_ERROR).into_response())
            },
        }
    }

    // pub async fn decor(db: Database) -> Fn {
    //     user_have_not_cookies_situation
    // }

    // pub async fn login(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
    //     println!("{session_hash}");
    //     Ok(warp::reply())
    // }
    
    // pub async fn register(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
    //     println!("{session_hash}");
    //     Ok(warp::reply())
    // }

    // pub async fn history(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
    //     println!("{session_hash}");
    //     Ok(warp::reply())
    // }
}
