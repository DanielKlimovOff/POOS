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
    // warp::serve(routes).run(([172, 17, 9, 223], 3030)).await;
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
    pub struct HistoryJson {
        pub history: Vec<Calculation>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct UsersJson {
        pub users: Vec<User>,
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
        pub role: String,
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
                .or(wrong_door())
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

    pub fn wrong_door() -> impl Filter<Extract = (impl warp::Reply,), Error = Infallible> + Clone {
        warp::any().map(|| {
            warp::redirect(warp::http::Uri::from_static("/"))
        })
    }

    pub fn api(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("api").and(
            calculate(db.clone())
            .or(delete_cookies(db.clone()))
            .or(login(db.clone()))
            .or(logout(db.clone()))
            .or(register(db.clone()))
            .or(history(db.clone()))
            .or(session_info(db.clone()))
            .or(get_users(db.clone()))
            .or(delete_user(db.clone()))
        )
    }

    pub fn delete_user(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("delete_user")
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::cookie("session_hash"))
            .and(warp::header("user_id"))
            .and(with_db(db))
            .and_then(handlers::delete_user)
    }

    pub fn get_users(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("get_users")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::cookie("session_hash"))
            .and(with_db(db))
            .and_then(handlers::get_users)
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

    pub fn register(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("register")
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::cookie("session_hash"))
            .and(json_body_login())
            .and(with_db(db))
            .and_then(handlers::register)
    }

    pub fn logout(db: Database) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("logout")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::cookie("session_hash"))
            .and(with_db(db))
            .and_then(handlers::create_new_session)
            
            // .map(|session_hash: String| {
            //     warp::reply::with_header(
            //         // warp::redirect(warp::http::Uri::from_static("/")),
            //         "LOGOUTED",
            //         "set-cookie",
            //         format!("session_hash=deleted; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT")).into_response()
            // })
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
            .or(users_page())
    }

    pub fn users_page() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("users")
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::fs::file("./data/users/users.html"))
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
                println!("cookies take {session_hash:?}");
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
    use crate::models::{CalculateJson, Calculation, Database, HistoryJson, Session, TestLoginJson, User, UsersJson};
    use http::Error;
    use warp::reply::Reply;
    use warp::http::StatusCode;
    use rusqlite::{params, types::Null};
    use rand;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use std::fs::File;
    use std::io::prelude::*;
    use base64::prelude::*;
    use md5;

    pub async fn calculate(session_hash: String, input_data: CalculateJson, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        println!("hash123 - {}", session_hash.clone());
        let mut result_data = CalculateJson {
            num1: input_data.num1,
            num2: input_data.num2,  
            operator_id: input_data.operator_id,
            result: None,
        };

        match input_data.operator_id {
            1 => result_data.result = Some(result_data.num1 + result_data.num2),
            2 => result_data.result = Some(result_data.num1 - result_data.num2),
            3 => result_data.result = Some(result_data.num1 * result_data.num2),
            4 => result_data.result = Some(result_data.num1 / result_data.num2),
            _ => result_data.result = None,
        }

        let session_info = get_session_info(db.clone(), session_hash).await;

        if let Err(mes) = session_info {
            println!("sql  {mes:?}");
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
                println!("sql another {massage:?}");
                Ok(warp::reply::with_status("ERROR_WITH_DB", StatusCode::INTERNAL_SERVER_ERROR).into_response())
            },
        }
    }

    pub async fn login(session_hash: String, login_data: TestLoginJson, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let user_info = get_user_info_by_login(db.clone(), login_data).await;

        println!("{user_info:?}");

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

    pub async fn register(session_hash: String, register_data: TestLoginJson, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let register_result = register_new_user(db.clone(), &register_data).await;

        println!("{register_result:?}");

        match register_result {
            Ok(_) => {
                let result = login(session_hash, register_data, db).await;
                match result {
                    Ok(ok) => Ok(ok.into_response()),
                    Err(rej) => Err(rej),
                }
            },
            // Err(mess) => Ok(warp::reply().into_response()),
            Err(mess) => Ok(warp::reply::with_status("ERROR WITH REGISTER", StatusCode::UNAUTHORIZED).into_response()),
        }
    }

    pub async fn get_users(session_hash: String, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let session_info = get_session_info(db.clone(), session_hash).await;
        if let Err(mes) = session_info {
            println!("{mes:?}");
            return Ok("USERS GET ERRROR1".into_response());
        }
        let session_info = session_info.unwrap();

        if session_info.is_auth == false {
            return Ok(warp::reply::with_status(warp::reply(), StatusCode::from_u16(228).unwrap()).into_response());
        }

        let users = get_users_from_db(db.clone()).await;
        if let Err(mes) = users {
            println!("{mes:?}");
            return Ok("USERS GET ERRROR2".into_response());
        }
        let users = users.unwrap();

        Ok(warp::reply::json(&users).into_response())
    }

    pub async fn delete_user(session_hash: String, user_id: i32, db: Database) -> Result<impl warp::Reply, warp::Rejection> {
        let session_info = get_session_info(db.clone(), session_hash).await;
        if let Err(mes) = session_info {
            println!("{mes:?}");
            return Ok("delete user ERRROR1".into_response());
        }
        let session_info = session_info.unwrap();

        if session_info.is_auth == false {
            return Ok(warp::reply::with_status(warp::reply(), StatusCode::from_u16(228).unwrap()).into_response());
        }

        let user_info = get_user_info_by_id(session_info.user_id.unwrap(), db.clone()).await;
        if let Err(mes) = user_info {
            println!("{mes:?}");
            return Ok("USERS GET ERRROR2".into_response());
        }
        let user_info = user_info.unwrap();

        if user_info.role != "moderling" {
            return Ok(warp::reply::with_status(warp::reply(), StatusCode::from_u16(229).unwrap()).into_response());
        }

        println!("uid delete user{user_id}");
        let db_response = db.lock().await.execute("delete from users where id = ?1;",
            [user_id]);
        match db_response {
            Ok(_) => Ok(warp::reply().into_response()),
            Err(massage) => Ok(warp::reply::with_status(massage.to_string(),
             StatusCode::INTERNAL_SERVER_ERROR).into_response()),
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

        let history;

        if session_info.is_auth {
            history = get_history_by_user_id(db.clone(), session_info.user_id.unwrap()).await;
        } else {
            history = get_history_by_session(db.clone(), session_info.id).await;
        }

        match history {
            Ok(history) => Ok(warp::reply::json(&history).into_response()),
            Err(err) => Ok(warp::reply::with_header(warp::reply::with_status("HISTORY ERROR", StatusCode::INTERNAL_SERVER_ERROR), "err message", err.to_string()).into_response())
        }
    }

    async fn get_history_by_session(db: Database, session_id: i32) -> Result<HistoryJson, rusqlite::Error> {
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
        Ok(HistoryJson { history })
    }

    async fn get_history_by_user_id(db: Database, user_id: i32) -> Result<HistoryJson, rusqlite::Error> {
        let db = db.lock().await;
        let mut stmt = db.prepare("select id, num1, num2, operator_id, result, session_id, user_id from calculations where user_id=?1")?;
        let history = stmt.query_map(params![user_id], |row| {
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
        Ok(HistoryJson { history })
    }

    async fn get_users_from_db(db: Database) -> Result<UsersJson, rusqlite::Error> {
        let db = db.lock().await;
        let mut stmt = db.prepare("select id, name, auth_hash, role from users")?;
        let data = stmt.query_map(params![], |row| {
            Ok(User{
                id: row.get(0)?,
                name: row.get(1)?,
                auth_hash: row.get(2)?,
                role: row.get(3)?,
            })
        })?;
        let data: Vec<User> = data.map(|e| e.unwrap()).collect();
        Ok(UsersJson { users: data })
    }

    async fn get_user_info_by_id(user_id: i32, db: Database) -> Result<User, rusqlite::Error> {
        let db_response = db.lock().await.query_row("select id, name, auth_hash, role from users where id = ?1", params![user_id], |row| {
            Ok(User{
                id: row.get(0)?,
                name: row.get(1)?,
                auth_hash: row.get(2)?,
                role: row.get(3)?,
            })
        });
        match db_response {
            Ok(user_info) => Ok(user_info),
            Err(massage) => Err(massage),
        }
    }

    async fn get_user_info_by_login(db: Database, login_data: TestLoginJson) -> Result<User, rusqlite::Error> {
        let auth_hash = login_data.name + ":" + &login_data.password;
        let db_response = db.lock().await.query_row("select id, name, auth_hash, role from users where auth_hash = ?1;", [&auth_hash],
         |row| Ok(User{
            id: row.get(0)?,
            name: row.get(1)?,
            auth_hash: row.get(2)?,
            role: row.get(3)?,
        }));
        match db_response {
            Ok(user_info) => Ok(user_info),
            Err(massage) => Err(massage),
        }
    }

    async fn register_new_user(db: Database, register_data: &TestLoginJson) -> Result<(), rusqlite::Error> {
        let auth_hash = register_data.clone().name + ":" + &register_data.password;
        let db_response = db.lock().await.execute("insert into users(name, auth_hash, role) values(?1, ?2, ?3);",
         [&register_data.name, &auth_hash, "normise"]);
        match db_response {
            Ok(_) => Ok(()),
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
        println!("you are in error situation {err:?}");

        create_new_session("".to_string(), db).await
    }

    pub async fn create_new_session(session_hash: String, db: Database) -> Result<impl warp::Reply, std::convert::Infallible> {
        let hash_seed = rand::random::<u32>();  
        let mut hasher = DefaultHasher::new();
        hash_seed.hash(&mut hasher);
        let new_session_hash = format!("{:x}", md5::compute(hasher.finish().to_string()));

        let mut file = File::open("data/names.txt").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let names: Vec<&str> = contents.split('\n').collect();
        let new_session_name = names[usize::try_from(hash_seed % 115).unwrap()].to_owned() + &(hash_seed % 100).to_string();
        let db_response = db.lock().await.execute("insert into sessions (hash, is_auth, name) values (?1, ?2, ?3);", params![&new_session_hash, false, &new_session_name]);
        
        match db_response {
            Ok(_) => Ok(warp::reply::with_header(
                warp::reply(),
                "set-cookie",
                format!("session_hash={new_session_hash}; path=/")).into_response()),
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
