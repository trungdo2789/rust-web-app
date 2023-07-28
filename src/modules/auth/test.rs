#[cfg(test)]
mod tests {
    use crate::{
        models::user::{self, User},
        modules::auth::{
            dto::{LoginDto, UserDto},
            service::{create_user, login, verify_jwt},
        },
        tests,
    };
    use mongodb::bson::doc;
    use std::time::Instant;

    #[actix_rt::test]
    async fn test_auth() {
        let now = Instant::now();
        let config = tests::utils::get_config();
        let db: mongodb::Database = tests::utils::get_database().await;
        let test_user = UserDto {
            email: "test@grindy.io".to_string(),
            first_name: "grindy".to_string(),
            last_name: "io".to_string(),
            password: "123".to_string(),
            username: now.elapsed().as_millis().to_string(),
        };
        let _ = create_user(test_user.clone(), db.clone()).await;
        let token = login(
            LoginDto {
                password: test_user.password.clone(),
                username: test_user.username.clone(),
            },
            config.jwt_secret.clone(),
            config.jwt_max_age.clone(),
            db.clone(),
        )
        .await;

        let id = verify_jwt(token.unwrap(), config.jwt_secret.clone()).unwrap();

        let db_user = db
            .collection::<User>(user::NAME)
            .find_one(Some(doc! {"_id": id}), None)
            .await
            .unwrap();

        match db_user {
            Some(u) => {
                let _ = db
                    .collection::<User>(user::NAME)
                    .delete_one(doc! {"_id": id}, None)
                    .await;
                assert_eq!(test_user.username.clone(), u.username.clone())
            }
            None => panic!("User doesn't exits"),
        }
    }
}
