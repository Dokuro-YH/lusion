use lusion_db::users::UserRepository;
use lusion_db::DbPool;
use tide::Context;

use crate::error::{EndpointResult, ResultExt};
use crate::response::{self, StatusCode};

pub async fn get_users<Pool>(cx: Context<Pool>) -> EndpointResult
where
    Pool: DbPool,
    Pool::Connection: UserRepository,
{
    let pool = cx.app_data();
    let users = pool.transaction(|conn| conn.find_users()).db_error()?;

    Ok(response::json(StatusCode::OK, users))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    fn app() -> tide::App<TestPool<PgPool>> {
        let pool = init_pool();
        let mut app = tide::App::new(pool);

        app.at("/users").get(get_users);
        app
    }

    #[test]
    fn test_get_users_should_be_200() {
        let mut server = init_service(app());
        let req = http::Request::get("/users").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "[]");
    }
}
