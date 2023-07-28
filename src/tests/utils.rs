use dotenv::from_filename;
use mongodb::Database;

use crate::{config::Config, db};

pub fn get_config() -> Config {
    from_filename(".env.test").ok();
    Config::init()
}

pub async fn get_database() -> Database {
    let config = get_config();
    db::get_database(config.mongodb_uri, config.mongodb_db_name).await
}

#[test]
fn test() {
    let config = get_config();
    assert_eq!(config.mongodb_db_name.as_str(), "test_rust_1")
}
