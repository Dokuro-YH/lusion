//! User repository
use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::DbError;
use crate::pg::PgConn;
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
    pub nickname: String,
    pub avatar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserPassword {
    pub old_password: String,
    pub new_password: String,
}

pub trait UserRepository {
    fn find_user(&self, user_id: &Uuid) -> Result<Option<User>, DbError>;

    fn find_users(&self) -> Result<Vec<User>, DbError>;

    fn create_user(&self, input: CreateUser) -> Result<User, DbError>;

    fn update_user_password(&self, user_id: &Uuid, new_password: &str) -> Result<usize, DbError>;

    fn delete_user(&self, user_id: &Uuid) -> Result<usize, DbError>;
}

impl UserRepository for PgConn {
    fn find_user(&self, user_id: &Uuid) -> Result<Option<User>, DbError> {
        use crate::schema::users::dsl::*;

        Ok(users.find(user_id).get_result::<User>(self).optional()?)
    }

    fn find_users(&self) -> Result<Vec<User>, DbError> {
        Ok(users::table.load::<User>(self)?)
    }

    fn create_user(&self, input: CreateUser) -> Result<User, DbError> {
        let id = Uuid::new_v4();
        let username = input.username;
        let password = input.password;
        let nickname = input.nickname;
        let avatar_url = input.avatar_url;
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
            .get_result(self)?)
    }

    fn update_user_password(&self, user_id: &Uuid, new_password: &str) -> Result<usize, DbError> {
        Ok(diesel::update(users::table.find(user_id))
            .set((
                users::password.eq(&new_password),
                users::updated_at.eq(&Utc::now()),
            ))
            .execute(self)?)
    }

    fn delete_user(&self, user_id: &Uuid) -> Result<usize, DbError> {
        Ok(diesel::delete(users::table.find(user_id)).execute(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_find_users_should_ok() {
        let result = with_transaction(|conn| conn.find_users());

        assert!(result.is_ok());
    }

    #[test]
    fn test_find_user_should_ok() {
        let result = with_transaction(|conn| conn.find_user(&Uuid::new_v4()));

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_user_should_ok() {
        let result = with_transaction(|conn| {
            conn.create_user(CreateUser {
                username: "admin".to_owned(),
                password: "1234".to_owned(),
                nickname: "admin",
                avatar_url: "empty.png",
            })
        });

        assert_matches!(result, Ok(user) => {
            assert_eq!(user.username, "admin");
            assert_eq!(user.nickname, "admin");
            assert_eq!(user.avatar_url, "empty.png");
        });
    }

    #[test]
    fn test_update_user_password_should_ok() {
        let result = with_transaction(|conn| conn.update_user_password(&Uuid::new_v4(), "4321"));

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_user_should_ok() {
        let result = with_transaction(|conn| conn.delete_user(&Uuid::new_v4()));

        assert!(result.is_ok());
    }
}
