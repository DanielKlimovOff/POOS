// #![deny(warnings)]

use serde_derive::{Deserialize, Serialize};

use warp::Filter;

#[derive(Serialize, Deserialize)]
struct MathOperation {
    value1: f64,
    value2: f64,
    operation: char,
}

#[derive(Serialize, Deserialize)]
struct MathResult {
    value: f64,
}

impl MathResult {
    pub fn new(result: f64) -> MathResult {
        MathResult {
            value: result,
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let site = warp::path::end()
        .and(warp::fs::file("./files/site.html"));

    let site_script = warp::path("script.js")
        .and(warp::fs::file("./files/script.js"));

    let wrong_door = warp::any()
        .and(warp::fs::file("./files/wrong_door.html"));

    let math = warp::post()
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|data: MathOperation| {
            let result;
            match data.operation {
                '+' => result = data.value1 + data.value2,
                '-' => result = data.value1 - data.value2,
                '*' => result = data.value1 * data.value2,
                '/' => result = data.value1 / data.value2,
                _ => panic!("GG"),
            }
            let result = (result * 100.0).round() / 100.0;
            let result = MathResult::new(result);
            warp::reply::json(&result)
        });


    let routes = warp::get().and(
        site
            .or(site_script)
            .or(wrong_door)
    ).or(
        math
    );

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}