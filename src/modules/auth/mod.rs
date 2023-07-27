use self::dto::{LoginDto, TokenClaims, UserDto};
use crate::{
    db::get_database,
    models::user::{self, User},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::{bson::doc, Database};

pub mod dto;
mod test;

/**
 * return entity id
 */
pub async fn create_user(user_dto: UserDto, db: Database) -> String {
    let user_collection = db.collection::<user::User>(user::NAME);
    let filter = doc! { "username": user_dto.username.clone() };
    let u: Option<user::User> = user_collection.find_one(filter, None).await.unwrap();
    match u {
        Some(_s) => panic!("User Exist"),
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
            let rs = user_collection
                .insert_one(entity.clone(), None)
                .await
                .unwrap();
            rs.inserted_id.to_string()
        }
    }
}

pub async fn login(params: LoginDto, jwt_secret: String, db: Database) -> Result<String, String> {
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

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
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

    match is_valid {
        true => Ok(token),
        false => Err("Invalid!".to_string()),
    }
}
