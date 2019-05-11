use lusion_db::prelude::*;
use lusion_db::users::{CreateUser, UserRepository};
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

pub async fn get_user<Pool>(cx: Context<Pool>) -> EndpointResult
where
    Pool: DbPool,
    Pool::Connection: UserRepository,
{
    let user_id = cx.param("user_id").user_error("Bad Request")?;
    let pool = cx.app_data();
    let user = pool
        .transaction(|conn| conn.find_user(&user_id))
        .db_error()?;
    let res = match user {
        Some(user) => response::json(StatusCode::OK, user),
        None => response::json(StatusCode::NOT_FOUND, json!({ "message": "Not Found" })),
    };

    Ok(res)
}

#[derive(Deserialize)]
struct PostUser {
    username: String,
    password: String,
    nickname: String,
}

pub async fn post_user<Pool>(mut cx: Context<Pool>) -> EndpointResult
where
    Pool: DbPool,
    Pool::Connection: UserRepository,
{
    let payload: PostUser = await!(cx.body_json()).user_error("Bad Request")?;
    let pool = cx.app_data();
    let username = payload.username;
    let password = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .user_error("password encode error")?;
    let nickname = payload.nickname;
    let avatar_url = random_avatar_url();
    let user = pool
        .transaction(|conn| {
            conn.create_user(CreateUser {
                username,
                password,
                nickname,
                avatar_url,
            })
        })
        .db_error()?;

    Ok(response::json(StatusCode::CREATED, user))
}

#[derive(Deserialize)]
struct PutPassword {
    old_password: String,
    new_password: String,
}

pub async fn put_user_password<Pool>(mut cx: Context<Pool>) -> EndpointResult
where
    Pool: DbPool,
    Pool::Connection: UserRepository,
{
    let user_id = cx.param("user_id").user_error("Bad Request")?;
    let payload: PutPassword = await!(cx.body_json()).user_error("Bad Request")?;
    let pool = cx.app_data();
    let user = pool.with(|conn| conn.find_user(&user_id)).db_error()?;

    let res = match user {
        None => response::json(StatusCode::NOT_FOUND, json!({ "message": "Not Found" })),
        Some(user) => {
            let verified =
                bcrypt::verify(&payload.old_password, &user.password).user_error("Bad Request")?;
            if verified {
                let password = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
                    .user_error("Bad Request")?;
                let _ = pool
                    .with(|conn| conn.update_user_password(&user_id, &password))
                    .db_error()?;
                response::empty(StatusCode::OK)
            } else {
                response::json(
                    StatusCode::BAD_REQUEST,
                    json!({ "message": "No match password" }),
                )
            }
        }
    };

    Ok(res)
}

pub async fn delete_user<Pool>(mut cx: Context<Pool>) -> EndpointResult
where
    Pool: DbPool,
    Pool::Connection: UserRepository,
{
    let user_id = cx.param("user_id").user_error("Bad Request")?;
    let pool = cx.app_data();
    let _ = pool.with(|conn| conn.delete_user(&user_id)).db_error()?;

    Ok(response::empty(StatusCode::NO_CONTENT))
}

fn random_avatar_url() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let avatar_num: i32 = rng.gen_range(1, 21);
    format!("/api/images/avatars/{}.png", avatar_num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    fn app() -> tide::App<TestPool<PgPool>> {
        let pool = init_pool();
        let mut app = tide::App::new(pool);

        app.at("/users").get(get_users);
        app.at("/users").post(post_user);
        app.at("/users/:user_id").get(get_user);
        app.at("/users/:user_id").delete(delete_user);
        app.at("/users/:user_id/password").put(put_user_password);

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

    #[test]
    fn test_get_user_should_be_404() {
        let mut server = init_service(app());
        let req = http::Request::get(format!("/users/{}", uuid::Uuid::new_v4())).to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 404);
        assert_eq!(res.read_body(), r#"{"message":"Not Found"}"#);
    }

    #[test]
    fn test_post_user_should_be_201() {
        let mut server = init_service(app());
        let payload = json!({
            "username": "testuser",
            "password": "1234",
            "nickname": "testname"
        });
        let req = http::Request::post("/users").json(payload);
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 201);
        let body = res.read_body();
        assert!(body.contains("username"));
        assert!(body.contains("testuser"));
        assert!(body.contains("testname"));
    }

    #[test]
    fn test_put_user_password_should_be_404() {
        let mut server = init_service(app());
        let payload = json!({
            "old_password": "1234",
            "new_password": "4321"
        });
        let req =
            http::Request::put(format!("/users/{}/password", uuid::Uuid::new_v4())).json(payload);
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 404);
    }

    #[test]
    fn test_delete_user_should_be_204() {
        let mut server = init_service(app());
        let req = http::Request::delete(format!("/users/{}", uuid::Uuid::new_v4())).to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 204);
    }
}
