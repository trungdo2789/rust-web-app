use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{HeaderName, HeaderValue},
        Method,
    },
    web::Data,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use log::{error, info};
use std::future::{ready, Ready};

use crate::{app_state::AppState, common::ResponseBody, constants, modules::auth};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct Authentication;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut authenticate_pass: bool = false;

        // Bypass some account routes
        let mut headers = req.headers().clone();
        headers.append(
            HeaderName::from_static("content-length"),
            HeaderValue::from_static("true"),
        );
        if Method::OPTIONS == *req.method() {
            authenticate_pass = true;
        } else {
            for ignore_route in constants::IGNORE_ROUTES.iter() {
                if req.path().starts_with(ignore_route) {
                    authenticate_pass = true;
                    break;
                }
            }
        }

        if !authenticate_pass {
            if let Some(app_state) = req.app_data::<Data<AppState>>() {
                if let Some(authen_header) = req.headers().get(constants::AUTHORIZATION) {
                    if let Ok(authen_str) = authen_header.to_str() {
                        if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
                            let token = authen_str[6..authen_str.len()].trim();
                            if let Ok(_id) = auth::service::verify_jwt(
                                token.to_string(),
                                app_state.config.jwt_secret.clone(),
                            ) {
                                authenticate_pass = true;
                            }
                        }
                    } else {
                        error!("Invalid token");
                    }
                }
            }
        }

        if !authenticate_pass {
            let (request, _pl) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(ResponseBody::new(
                    constants::MESSAGE_INVALID_TOKEN,
                    constants::EMPTY_STR,
                    false,
                ))
                .map_into_right_body();

            return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
        }

        let res = self.service.call(req);
        Box::pin(async move { res.await.map(ServiceResponse::map_into_left_body) })
    }

    forward_ready!(service);
}
