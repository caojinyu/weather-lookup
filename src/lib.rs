use http_req::request;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use webhook_flows::{create_endpoint, request_handler, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(
    _headers: Vec<(String, String)>,
    subpath: String,
    qry: HashMap<String, Value>,
    body: Vec<u8>,
) {
    let city = qry.get("city").unwrap_or(&Value::Null).as_str();

    let resp = match city {
        Some(c) => get_weather(c).map(|w| {
            format!(
                r#"
        城市: {}
        最低温度: {} °C
        最高温度 {} °C
        风速 {} km/h
        "#,
                w.weather
                    .first()
                    .unwrap_or(&Weather {
                        main: "Unknown".to_string()
                    })
                    .main,
                w.main.temp_min as i32,
                w.main.temp_max as i32,
                w.wind.speed as i32
            )
        }),
        None => Err(String::from("No city in query")),
    };

    match resp {
        Ok(r) => send_response(
            200,
            vec![(
                String::from("content-type"),
                String::from("text/html;charset=UTF-8"),
            )],
            r.as_bytes().to_vec(),
        ),
        Err(e) => send_response(
            400,
            vec![(
                String::from("content-type"),
                String::from("text/html;charset=UTF-8"),
            )],
            e.as_bytes().to_vec(),
        ),
    }
}

#[derive(Deserialize)]
struct ApiResult {
    weather: Vec<Weather>,
    main: Main,
    wind: Wind,
}

#[derive(Deserialize)]
struct Weather {
    main: String,
}

#[derive(Deserialize)]
struct Main {
    temp_max: f64,
    temp_min: f64,
}

#[derive(Deserialize)]
struct Wind {
    speed: f64,
}

fn get_weather(city: &str) -> Result<ApiResult, String> {
    let mut writer = Vec::new();
    let api_key = "92455ca4c3d48f544e55950af2da5cea";
    let query_str = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={city}&units=metric&appid={api_key}"
    );
    request::get(query_str, &mut writer)
        .map_err(|e| e.to_string())
        .and_then(|_| {
            serde_json::from_slice::<ApiResult>(&writer).map_err(|_| {
                "
            请检查准确的城市"
                    .to_string()
            })
        })
}
