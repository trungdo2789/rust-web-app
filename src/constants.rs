// ignore routes
pub const IGNORE_ROUTES: [&str; 3] = ["/api/ping", "/api/auth/signup", "/api/auth/login"];
// Default number of items per page
pub const DEFAULT_PER_PAGE: i64 = 10;

// Default page number
pub const DEFAULT_PAGE_NUM: i64 = 1;

pub const EMPTY_STR: &str = "";

// Headers
pub const AUTHORIZATION: &str = "Authorization";

pub const MESSAGE_INVALID_TOKEN: &str = "Invalid token, please login again";