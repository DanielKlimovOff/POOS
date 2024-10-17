use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=site=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "site=info");
    }
    pretty_env_logger::init();
    
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
    pub struct TestLoginJson {
        pub name: String,
        pub password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub id: i32,
        pub name: String,
        pub auth_hash: String,   
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Session {
        pub id: i32,
        pub hash: String,
        pub is_auth: bool,   
        pub user_id: Option<i32>,
        pub name: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Calculation {
        pub id: i32,
        pub num1: f64,
        pub num2: f64,
        pub operator_id: i32,
        pub result: f64,
        pub session_id: i32,
        pub user_id: Option<i32>,
    }

    pub fn open_db(name_db: &str) -> Database {
        let db = Connection::open(name_db).unwrap();
        let db = Arc::new(Mutex::new(db));
        db
    }
}

mod filters {
    use std::convert::Infallible;

    use crate::{handlers, models};
    use warp::http::StatusCode;
    use warp::{reply::Reply, Filter};
    use crate::models::{CalculateJson, Database, TestLoginJson};

    pub fn site(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone {
        check_cookies()
            .untuple_one()
            .and(
                api(db.clone())
                .or(data())
                .or(pages())
            )
            .recover(  move |err|  {
                handlers::user_have_not_cookies_situation(db.clone(), err)
            } )
            // .map(|_, r| r)

        // warp::path("test")
        //     .and(warp::path::param())
        //     .and_then(|s: String| async {
        //         if s == "abc" {
        //             Ok(warp::path("end")
        //                 .map(|| warp::reply()).boxed())
        //         } else {
        //             Ok(warp::path("noabc")
        //                 .map(|| warp::reply()).boxed())
        //         }
        //     })
    }

    pub fn api(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("api").and(
            calculate(db.clone())
            .or(delete_cookies(db.clone()))
            .or(login(db.clone()))
            .or(logout())
            .or(history(db.clone()))
            .or(session_info(db.clone()))
        )
    }

    pub fn calculate(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("calculate")
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::cookie("session_hash"))
            .and(json_body_calculate())
            .and(with_db(db))
            .and_then(handlers::calculate)
    }

    pub fn login(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("login")
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::cookie("session_hash"))
            .and(json_body_login())
            .and(with_db(db))
            .and_then(handlers::login)
    }

    pub fn logout() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("logout")
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::cookie("session_hash"))
            .map(|session_hash: String| {
                warp::reply::with_header(
                    warp::reply(),
                    "set-cookie",
                    format!("session_hash=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT")).into_response()
            })
    }

    pub fn session_info(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("session_info")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::cookie("session_hash"))
            .and(with_db(db))
            .and_then(handlers::session_info)
    }

    pub fn history(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("history")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::cookie("session_hash"))
            .and(with_db(db))
            .and_then(handlers::history)
    }

    pub fn delete_cookies(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("delete_cookies")
            .and(warp::path::end())
            .and(warp::get())
            .and(with_db(db))
            .map(|db| {
                warp::reply::with_header(
                    warp::reply(),
                    "set-cookie",
                    format!("session_hash=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT")).into_response()
            })
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

    fn json_body_login() -> impl Filter<Extract = (TestLoginJson,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    fn check_cookies() -> impl Filter<Extract = ((), ), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::any()
            .and(warp::cookie::optional("session_hash"))
            .and_then(|session_hash: Option<String>| async move {
                if session_hash == None{
                    return Err(warp::reject::custom(models::UnIdentified));
                }
                Ok(())
            })
    }

    // fn catch_header(db: Database) -> impl Filter<Extract = (_, ), Error = std::convert::Infallible> + Clone {
    //     warp::header("session_hash")
    // }
}

mod handlers {
    use crate::models::{CalculateJson, Database, Session, TestLoginJson, User, Calculation};
    use http::Error;
    use warp::reply::Reply;
    use warp::http::StatusCode;
    use rusqlite::{params, types::Null};
    use rand;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    pub async fn calculate(session_hash: String, input_data: CalculateJson, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        println!("hash123 - {}", session_hash.clone());
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

        let session_info = get_session_info(db.clone(), session_hash).await;

        if let Err(mes) = session_info {
            println!("{mes:?}");
            return Ok("SESSION GET ERRROR not found session".into_response());
        }

        let session_info = session_info.unwrap();

        let db_response = db.lock().await.execute("insert into calculations (num1, num2, operator_id, result, session_id, user_id) values (?1, ?2, ?3, ?4, ?5, ?6);", params![
            &result_data.num1,
            &result_data.num2,
            &result_data.operator_id,
            &result_data.result,
            session_info.id,
            session_info.user_id,
        ]);
        
        match db_response {
            Ok(_) => Ok(warp::reply::json(&result_data).into_response()),
            Err(massage) => {
                println!("{massage:?}");
                Ok(warp::reply::with_status("ERROR_WITH_DB", StatusCode::INTERNAL_SERVER_ERROR).into_response())
            },
        }
    }

    pub async fn login(session_hash: String, login_data: TestLoginJson, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let user_info = get_player_info_by_login(db.clone(), login_data).await;

        match user_info {
            Ok(user_info) => {
                let db_response = db.lock().await.execute("update sessions set is_auth=true, user_id=?1, name=?2 where hash=?3;", params![
                user_info.id,
                &user_info.name,
                session_hash,
            ]);
            
            match db_response {
                Ok(_) => Ok(warp::reply().into_response()),
                Err(massage) => {
                    println!("{massage:?}");
                    Ok(warp::reply::with_status("ERROR_WITH_DB", StatusCode::INTERNAL_SERVER_ERROR).into_response())
                },
            }
            },
            Err(mess) => Ok(warp::reply::with_status("USER NOT EXIST", StatusCode::UNAUTHORIZED).into_response()),
        }
    }

    // pub async fn logout(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
    //     let db_response = db.lock().await.execute("update sessions set is_auth=false, user_id=?1, name=?2 where hash=?3;", params![
    //         user_info.id,
    //         &user_info.name,
    //         session_hash,
    //     ])
        
    //     match db_response {
    //         Ok(_) => Ok(warp::reply().into_response()),
    //         Err(massage) => {
    //             println!("{massage:?}");
    //             Ok(warp::reply::with_status("ERROR_WITH_DB", StatusCode::INTERNAL_SERVER_ERROR).into_response())
    //         },
    //     }
    // }

    pub async fn session_info(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let session_info = get_session_info(db.clone(), session_hash).await;
        if let Err(mes) = session_info {
            println!("{mes:?}");
            return Ok("SESSION GET ERRROR".into_response());
        }
        let session_info = session_info.unwrap();

        Ok(warp::reply::json(&session_info).into_response())
    }

    pub async fn history(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let session_info = get_session_info(db.clone(), session_hash).await;
        if let Err(mes) = session_info {
            println!("{mes:?}");
            return Ok("SESSION GET ERRROR".into_response());
        }
        let session_info = session_info.unwrap();

        if session_info.is_auth {

        } else {
            get_history_by_session(db.clone(), session_info.id);
        }

        Ok(warp::reply::json(&session_info).into_response())
    }

    async fn get_history_by_session(db: Database, session_id: i32) -> Result<Vec<Calculation>, rusqlite::Error> {
        let db = db.lock().await;
        let mut stmt = db.prepare("select id, num1, num2, operator_id, result, session_id, user_id from calculations where session_id=?1")?;
        let history = stmt.query_map(params![session_id], |row| {
            Ok(Calculation {
                id: row.get(0)?,
                num1: row.get(1)?,
                num2: row.get(2)?,
                operator_id: row.get(3)?,
                result: row.get(4)?,
                session_id: row.get(5)?,
                user_id: row.get(6)?,
            })
        })?;
        let history: Vec<Calculation> = history.map(|e| e.unwrap()).collect();
        Ok(history)
    }

    async fn get_player_info_by_login(db: Database, login_data: TestLoginJson) -> Result<User, rusqlite::Error> {
        let auth_hash = login_data.name + ":" + &login_data.password;
        let db_response = db.lock().await.query_row("select id, name, auth_hash from users where auth_hash = ?1;", [&auth_hash],
         |row| Ok(User{
            id: row.get(0)?,
            name: row.get(1)?,
            auth_hash: row.get(2)?,
        }));
        match db_response {
            Ok(user_info) => Ok(user_info),
            Err(massage) => Err(massage),
        }
    }

    

    async fn get_session_info(db: Database, session_hash: String) -> Result<Session, rusqlite::Error> {
        let db_response = db.lock().await.query_row("select id, hash, is_auth, user_id, name from sessions where hash = ?1;", [&session_hash], |row| Ok(Session{
                id: row.get(0)?,
                hash: row.get(1)?,
                is_auth: row.get(2)?,
                user_id: row.get(3)?,
                name: row.get(4)?,
            }));
        match db_response {
            Ok(session_info) => Ok(session_info),
            Err(massage) => Err(massage),
            // Error(error_massage) => Err(warp::reject::custom(error_on_db)),
        }
    }

    pub async fn user_have_not_cookies_situation(db: Database, err: warp::Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
        let hash_seed = rand::random::<u32>();  
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
