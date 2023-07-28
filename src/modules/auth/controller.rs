use std::str::FromStr;
use utoipa::OpenApi;

use crate::{
    app_state::AppState,
    common::ResponseBody,
    constants::EMPTY_STR,
    middleware::authentication::Authentication,
    modules::auth::dto::{LoginDto, LoginResponse, UserDto},
};
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{Data, Json},
    Error as AWError, HttpResponse, Result,
};

use serde_json::json;

use super::service;

#[utoipa::path(
    request_body = LoginDto,
    responses(
        (status = 200, description = "Success", body = [ResponseBody<LoginResponse>]),
        (status = 401, description = "Unauthorize", body = [ResponseBody<&str>])
    ),
    tag = "auth"
)]
#[post("/login")]
pub async fn login(
    app_state: Data<AppState>,
    Json(item): Json<LoginDto>,
) -> Result<HttpResponse, AWError> {
    let result = service::login(
        item,
        app_state.config.jwt_secret.clone(),
        app_state.config.jwt_max_age.clone(),
        app_state.db.clone(),
    )
    .await;
    match result {
        Ok(jwt) => Ok(HttpResponse::build(StatusCode::OK).json(
            ResponseBody::<LoginResponse>::new(
                EMPTY_STR,
                LoginResponse { access_token: jwt },
                true,
            ),
        )),
        Err(e) => Ok(
            HttpResponse::build(StatusCode::from_str(e.status.as_str()).unwrap()).json(
                ResponseBody::<&str>::new(e.message.to_string().as_str(), EMPTY_STR, false),
            ),
        ),
    }
}

#[utoipa::path(
    request_body = UserDto,
    responses(
        (status = 200, description = "Success", body = [ResponseBody::<&str>]),
    ),
    tag = "auth"
)]
#[post("/create")]
pub async fn create(
    app_state: Data<AppState>,
    Json(item): Json<UserDto>,
) -> Result<HttpResponse, AWError> {
    let result = service::create_user(item, app_state.db.clone()).await;
    match result {
        Ok(id) => Ok(HttpResponse::build(StatusCode::OK)
            .json(ResponseBody::<String>::new(EMPTY_STR, id, true))),
        Err(e) => Ok(HttpResponse::build(StatusCode::OK)
            .json(ResponseBody::<&str>::new(&e.message, EMPTY_STR, false))),
    }
}

#[get("/me", wrap = "Authentication")]
pub async fn get_me(
    app_state: Data<AppState>,
    Json(item): Json<UserDto>,
) -> Result<HttpResponse, AWError> {
    let result = service::create_user(item, app_state.db.clone()).await;
    match result {
        Ok(id) => Ok(HttpResponse::build(StatusCode::OK).body(json!({ "id": id }).to_string())),
        Err(e) => {
            println!("{}", json!(e));
            return Ok(
                HttpResponse::build(StatusCode::from_str(e.status.as_str()).unwrap())
                    .body(json!(e).to_string()),
            );
        }
    }
}

/// returns the endpoints for the Auth service
pub fn endpoints(scope: actix_web::Scope) -> actix_web::Scope {
    scope.service(login).service(create).service(get_me)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        login,
        create
    ),
    components(
        schemas(
            ResponseBody<LoginResponse>, 
            LoginResponse, 
            LoginDto, 
            UserDto
        )
    ),
    tags(
        (name = "auth", description = "auth API")
    ),
    servers(
        (url = "/auth")
    )
)]
pub struct ApiDoc;
