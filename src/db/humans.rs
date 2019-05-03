//! Human database access.
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::PgConn;
use crate::error::{Result, ResultExt};
use crate::schema::{human_friends, humans};

#[derive(Debug, PartialEq, Queryable)]
pub struct Human {
    pub id: Uuid,
    pub name: String,
}

#[derive(GraphQLInputObject)]
pub struct CreateHuman {
    pub name: String,
    pub friend_ids: Vec<Uuid>,
}

#[derive(GraphQLInputObject)]
pub struct UpdateHuman {
    pub name: String,
    pub friend_ids: Vec<Uuid>,
}

#[derive(Insertable)]
#[table_name = "human_friends"]
struct HumanFriend<'a> {
    human_id: &'a Uuid,
    friend_id: &'a Uuid,
}

pub trait HumanRepository {
    fn find_humans(&self) -> Result<Vec<Human>>;

    fn find_human(&self, id: &Uuid) -> Result<Option<Human>>;

    fn create_human(&self, input: CreateHuman) -> Result<Human>;

    fn update_human(&self, human_id: &Uuid, input: UpdateHuman) -> Result<Option<Human>>;

    fn delete_human(&self, human_id: &Uuid) -> Result<usize>;

    fn find_friends_by_human_id(&self, human_id: &Uuid) -> Result<Vec<Human>>;
}

impl HumanRepository for PgConn {
    fn find_humans(&self) -> Result<Vec<Human>> {
        use crate::schema::humans::dsl::*;
        let conn = self.get_conn();

        Ok(humans.load(conn).db_error()?)
    }

    fn find_human(&self, id: &Uuid) -> Result<Option<Human>> {
        let conn = self.get_conn();
        Ok(humans::table
            .find(id)
            .get_result(conn)
            .optional()
            .db_error()?)
    }

    fn create_human(&self, input: CreateHuman) -> Result<Human> {
        use crate::schema::humans::dsl::*;
        let conn = self.get_conn();

        let human_id = Uuid::new_v4();
        let human = diesel::insert_into(humans)
            .values((id.eq(&human_id), name.eq(&input.name)))
            .get_result::<Human>(conn)
            .db_error()?;

        let friends = input
            .friend_ids
            .iter()
            .map(|friend_id| HumanFriend {
                human_id: &human.id,
                friend_id,
            })
            .collect::<Vec<HumanFriend>>();
        diesel::insert_into(human_friends::table)
            .values(&friends)
            .execute(conn)
            .db_error()?;

        Ok(human)
    }

    fn update_human(&self, human_id: &Uuid, input: UpdateHuman) -> Result<Option<Human>> {
        use crate::schema::humans::dsl::*;
        let conn = self.get_conn();

        let human = diesel::update(humans.find(human_id))
            .set(name.eq(&input.name))
            .get_result::<Human>(conn)
            .optional()
            .db_error()?;

        match human {
            None => Ok(None),
            Some(human) => {
                let _ = diesel::delete(human_friends::table)
                    .filter(human_friends::human_id.eq(human_id))
                    .execute(conn)
                    .db_error()?;
                let friends = input
                    .friend_ids
                    .iter()
                    .map(|friend_id| HumanFriend {
                        human_id: &human.id,
                        friend_id,
                    })
                    .collect::<Vec<HumanFriend>>();
                diesel::insert_into(human_friends::table)
                    .values(&friends)
                    .execute(conn)
                    .db_error()?;
                Ok(Some(human))
            }
        }
    }

    fn delete_human(&self, human_id: &Uuid) -> Result<usize> {
        use crate::schema::humans::dsl::*;
        let conn = self.get_conn();

        let _ = diesel::delete(human_friends::table)
            .filter(human_friends::friend_id.eq(human_id))
            .execute(conn)
            .db_error()?;
        let _ = diesel::delete(human_friends::table)
            .filter(human_friends::human_id.eq(human_id))
            .execute(conn)
            .db_error()?;
        let updated = diesel::delete(humans.find(human_id))
            .execute(conn)
            .db_error()?;

        Ok(updated)
    }

    fn find_friends_by_human_id(&self, human_id: &Uuid) -> Result<Vec<Human>> {
        use diesel::dsl::any;
        let conn = self.get_conn();

        let friend_ids = human_friends::table
            .select(human_friends::friend_id)
            .filter(human_friends::human_id.eq(human_id))
            .load::<Uuid>(conn)
            .db_error()?;

        Ok(humans::table
            .filter(humans::id.eq(any(friend_ids)))
            .load(conn)
            .db_error()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_find_human_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.find_human(&Uuid::new_v4()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_humans_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.find_humans());
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_human_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| {
            let alice = conn.create_human(CreateHuman {
                name: "alice".to_owned(),
                friend_ids: vec![],
            })?;

            let bob = conn.create_human(CreateHuman {
                name: "bob".to_owned(),
                friend_ids: vec![alice.id],
            })?;

            let bob_friends = conn.find_friends_by_human_id(&bob.id)?;

            Ok((bob, bob_friends, alice))
        });

        assert_matches!(result, Ok((bob, bob_friends, alice)) => {
            assert_eq!(bob.name, "bob");
            assert_eq!(alice.name, "alice");
            assert_eq!(bob_friends, vec![alice]);
        });
    }

    #[test]
    fn test_update_human_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| {
            let old_bob = conn.create_human(CreateHuman {
                name: "old_bob".to_owned(),
                friend_ids: vec![],
            })?;
            let old_bob_friends = conn.find_friends_by_human_id(&old_bob.id)?;

            let alice = conn.create_human(CreateHuman {
                name: "alice".to_owned(),
                friend_ids: vec![],
            })?;

            let new_bob = conn.update_human(
                &old_bob.id,
                UpdateHuman {
                    name: "new_bob".to_owned(),
                    friend_ids: vec![alice.id],
                },
            )?;
            assert!(new_bob.is_some());
            let new_bob = new_bob.unwrap();
            let new_bob_friends = conn.find_friends_by_human_id(&new_bob.id)?;

            Ok((old_bob, old_bob_friends, new_bob, new_bob_friends, alice))
        });

        assert_matches!(result, Ok((old_bob, old_bob_friends, new_bob, new_bob_friends, alice)) => {
            assert_eq!(old_bob.name, "old_bob");
            assert_eq!(new_bob.name, "new_bob");
            assert_eq!(alice.name, "alice");
            assert_eq!(old_bob_friends, vec![]);
            assert_eq!(new_bob_friends, vec![alice]);
        })
    }

    #[test]
    fn test_delete_human_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.delete_human(&Uuid::new_v4()));

        assert!(result.is_ok());
    }

    #[test]
    fn test_find_friends_by_human_id_should_ok() {
        let pool = init_pool();

        let result = pool.test_transaction(|conn| conn.find_friends_by_human_id(&Uuid::new_v4()));

        assert!(result.is_ok());
    }

}
