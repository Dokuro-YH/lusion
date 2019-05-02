//! User repository
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::prelude::*;
use diesel::prelude::*;
use rand::Rng;
use uuid::Uuid;

use crate::db::PgConn;
use crate::error::{self, Result, ResultExt};
use crate::schema::users;

#[derive(Debug, PartialEq, Queryable, Insertable, Serialize)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub nickname: String,
    pub avatar_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserPassword {
    pub old_password: String,
    pub new_password: String,
}

pub trait UserRepository {
    fn find_user(&self, user_id: &Uuid) -> Result<Option<User>>;

    fn find_users(&self) -> Result<Vec<User>>;

    fn create_user(&self, input: CreateUser) -> Result<User>;

    fn update_user_password(
        &self,
        user_id: &Uuid,
        input: UpdateUserPassword,
    ) -> Result<Option<User>>;

    fn delete_user(&self, user_id: &Uuid) -> Result<usize>;
}

impl UserRepository for PgConn {
    fn find_user(&self, user_id: &Uuid) -> Result<Option<User>> {
        use crate::schema::users::dsl::*;

        let conn = self.get_conn();

        Ok(users
            .find(user_id)
            .get_result::<User>(conn)
            .optional()
            .db_error()?)
    }

    fn find_users(&self) -> Result<Vec<User>> {
        let conn = self.get_conn();

        Ok(users::table.load::<User>(conn).db_error()?)
    }

    fn create_user(&self, input: CreateUser) -> Result<User> {
        let conn = self.get_conn();
        let id = Uuid::new_v4();
        let username = input.username;
        let password = hash(&input.password, DEFAULT_COST).user_error("password encode error")?;
        let nickname = input.nickname.unwrap_or_else(|| username.clone());
        let avatar_url = input.avatar_url.unwrap_or_else(random_avatar_url);
        let now = Utc::now();

        Ok(diesel::insert_into(users::table)
            .values(User {
                id,
                username,
                password,
                nickname,
                avatar_url,
                created_at: now,
                updated_at: now,
            })
            .get_result(conn)
            .db_error()?)
    }

    fn update_user_password(
        &self,
        user_id: &Uuid,
        input: UpdateUserPassword,
    ) -> Result<Option<User>> {
        let conn = self.get_conn();

        if let Some(mut user) = self.find_user(user_id)? {
            let verified =
                verify(&input.old_password, &user.password).user_error("password encode error")?;

            if verified {
                let hashed_password =
                    hash(&input.new_password, DEFAULT_COST).user_error("password encode error")?;

                user.password = hashed_password;
                user.updated_at = Utc::now();

                diesel::update(users::table.find(user_id))
                    .set((
                        users::password.eq(&user.password),
                        users::updated_at.eq(&user.updated_at),
                    ))
                    .execute(conn)
                    .db_error()?;

                return Ok(Some(user));
            } else {
                return Err(error::user_error("password not match"));
            }
        }

        Ok(None)
    }

    fn delete_user(&self, user_id: &Uuid) -> Result<usize> {
        let conn = self.get_conn();

        Ok(diesel::delete(users::table.find(user_id))
            .execute(conn)
            .db_error()?)
    }
}

pub fn random_avatar_url() -> String {
    let mut rng = rand::thread_rng();
    let avatar_num: i32 = rng.gen_range(1, 21);
    format!("/api/images/avatars/{}.png", avatar_num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn find_users_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.find_users());

        assert!(result.is_ok());
    }

    #[test]
    fn find_user_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.find_user(&Uuid::new_v4()));

        assert!(result.is_ok());
    }

    #[test]
    fn create_user_should_return_user() {
        let pool = init_pool();
        let result = pool.test_transaction(|conn| {
            conn.create_user(CreateUser {
                username: "admin".to_owned(),
                password: "1234".to_owned(),
                nickname: None,
                avatar_url: None,
            })
        });

        assert_matches!(result, Ok(user) => {
            assert_eq!(user.username, "admin");
            assert_eq!(user.nickname, "admin");
        });
    }

    #[test]
    fn update_user_password_should_ok() {
        let pool = init_pool();
        let result = pool.test_transaction(|conn| {
            let user = conn.create_user(CreateUser {
                username: "admin".to_owned(),
                password: "1234".to_owned(),
                nickname: None,
                avatar_url: None,
            })?;

            conn.update_user_password(
                &user.id,
                UpdateUserPassword {
                    old_password: "1234".to_owned(),
                    new_password: "4321".to_owned(),
                },
            )
        });

        assert!(result.is_ok());
    }

    #[test]
    fn update_user_password_should_be_password_not_match_err() {
        let pool = init_pool();
        let result = pool.test_transaction(|conn| {
            let user = conn.create_user(CreateUser {
                username: "admin".to_owned(),
                password: "1234".to_owned(),
                nickname: None,
                avatar_url: None,
            })?;

            conn.update_user_password(
                &user.id,
                UpdateUserPassword {
                    old_password: "not_match".to_owned(),
                    new_password: "4321".to_owned(),
                },
            )
        });

        assert_matches!(result, Err(err) => {
            assert_eq!(err.kind(), error::user_error("password not match").kind());
        });
    }

    #[test]
    fn delete_user_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.delete_user(&Uuid::new_v4()));

        assert!(result.is_ok());
    }
}
