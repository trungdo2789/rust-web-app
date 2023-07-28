use std::str::FromStr;

use crate::{
    common::ErrorResponse,
    models::user::{self, User},
};

use actix_web::http;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};

use super::dto::{LoginDto, TokenClaims, UserDto};

/**
 * return entity id
 */
pub async fn create_user(user_dto: UserDto, db: Database) -> Result<String, ErrorResponse> {
    let user_collection = db.collection::<user::User>(user::NAME);
    let filter = doc! { "username": user_dto.username.clone() };
    let u: Option<user::User> = user_collection.find_one(filter, None).await.unwrap();
    match u {
        Some(_s) => Err(ErrorResponse {
            message: "User exist".to_string(),
            status: http::StatusCode::BAD_REQUEST.as_u16().to_string(),
        }),
        None => {
            let salt = SaltString::generate(&mut OsRng);
            let hashed_password = Argon2::default()
                .hash_password(user_dto.password.as_bytes(), &salt)
                .expect("Error while hashing password")
                .to_string();
            let entity = User {
                id: None,
                username: user_dto.username,
                email: user_dto.email,
                first_name: user_dto.first_name,
                last_name: user_dto.last_name,
                password: hashed_password,
                salt_nonce: salt.to_string(),
            };
            let rs = user_collection.insert_one(entity.clone(), None).await;

            match rs {
                Ok(r) => Ok(r.inserted_id.to_string()),
                Err(e) => Err(ErrorResponse {
                    message: e.to_string(),
                    status: http::StatusCode::INTERNAL_SERVER_ERROR.as_u16().to_string(),
                }),
            }
        }
    }
}

pub async fn login(
    params: LoginDto,
    jwt_secret: String,
    jwt_expired: i32,
    db: Database,
) -> Result<String, ErrorResponse> {
    let user_db = db
        .collection::<user::User>(user::NAME)
        .find_one(
            doc! {
              "username": params.username.clone()
            },
            None,
        )
        .await
        .unwrap();

    let is_valid = user_db.clone().map_or(false, |user| {
        let parsed_hash = PasswordHash::new(&user.password).unwrap();
        Argon2::default()
            .verify_password(params.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true)
    });

    match is_valid {
        true => {
            let now = Utc::now();
            let iat = now.timestamp() as usize;
            let exp = (now + Duration::minutes(jwt_expired as i64)).timestamp() as usize;
            let claims: TokenClaims = TokenClaims {
                sub: user_db.clone().unwrap().id.unwrap().to_string(),
                exp,
                iat,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(jwt_secret.as_ref()),
            )
            .unwrap();
            Ok(token)
        }
        false => Err(ErrorResponse {
            message: "Unauthorize".to_string(),
            status: "401".to_string(),
        }),
    }
}

pub fn verify_jwt(token: String, jwt_secret: String) -> Result<ObjectId, ErrorResponse> {
    let claims = decode::<TokenClaims>(
        &token.as_str(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    );
    match claims {
        Ok(c) => {
            let id = ObjectId::from_str(c.claims.sub.as_str()).unwrap();
            return Ok(id);
        }
        Err(e) => Err(ErrorResponse {
            message: e.to_string(),
            status: "401".to_string(),
        }),
    }
}
