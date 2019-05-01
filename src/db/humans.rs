//! Human database access.
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::PgConn;
use crate::error::{ErrorKind, Result, ResultExt};
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

        Ok(humans.load(conn).context(ErrorKind::DbError)?)
    }

    fn find_human(&self, id: &Uuid) -> Result<Option<Human>> {
        let conn = self.get_conn();
        Ok(humans::table
            .find(id)
            .first(conn)
            .optional()
            .context(ErrorKind::DbError)?)
    }

    fn create_human(&self, input: CreateHuman) -> Result<Human> {
        use crate::schema::humans::dsl::*;
        let conn = self.get_conn();

        let human_id = Uuid::new_v4();
        let human = diesel::insert_into(humans)
            .values((id.eq(&human_id), name.eq(&input.name)))
            .get_result::<Human>(conn)
            .context(ErrorKind::DbError)?;

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
            .context(ErrorKind::DbError)?;

        Ok(human)
    }

    fn update_human(&self, human_id: &Uuid, input: UpdateHuman) -> Result<Option<Human>> {
        use crate::schema::humans::dsl::*;
        let conn = self.get_conn();

        let human = diesel::update(humans.find(human_id))
            .set(name.eq(&input.name))
            .get_result::<Human>(conn)
            .optional()
            .context(ErrorKind::DbError)?;

        match human {
            None => Ok(None),
            Some(human) => {
                let _ = diesel::delete(human_friends::table)
                    .filter(human_friends::human_id.eq(human_id))
                    .execute(conn)
                    .context(ErrorKind::DbError)?;
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
                    .context(ErrorKind::DbError)?;
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
            .context(ErrorKind::DbError)?;
        let _ = diesel::delete(human_friends::table)
            .filter(human_friends::human_id.eq(human_id))
            .execute(conn)
            .context(ErrorKind::DbError)?;
        let updated = diesel::delete(humans.find(human_id))
            .execute(conn)
            .context(ErrorKind::DbError)?;

        Ok(updated)
    }

    fn find_friends_by_human_id(&self, human_id: &Uuid) -> Result<Vec<Human>> {
        use diesel::dsl::any;
        let conn = self.get_conn();

        let friend_ids = human_friends::table
            .select(human_friends::friend_id)
            .filter(human_friends::human_id.eq(human_id))
            .load::<Uuid>(conn)
            .context(ErrorKind::DbError)?;

        Ok(humans::table
            .filter(humans::id.eq(any(friend_ids)))
            .load(conn)
            .context(ErrorKind::DbError)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn find_human_should_return_none() {
        let pool = init_pool();

        let human = pool
            .test_transaction(|conn| conn.find_human(&Uuid::new_v4()))
            .unwrap();
        assert_eq!(human, None);
    }

    #[test]
    fn find_humans_should_return_empty_vec() {
        let pool = init_pool();

        let humans = pool.test_transaction(|conn| conn.find_humans()).unwrap();
        assert_eq!(humans, Vec::<Human>::new());
    }

    #[test]
    fn create_human_should_return_human_name() {
        let pool = init_pool();

        let human = pool
            .test_transaction(|conn| {
                conn.create_human(CreateHuman {
                    name: "bob".to_owned(),
                    friend_ids: vec![],
                })
            })
            .unwrap();

        assert_eq!(human.name, "bob");
    }

    #[test]
    fn create_human_should_add_friends() {
        let pool = init_pool();

        let friends = pool
            .test_transaction(|conn| {
                let bob = conn.create_human(CreateHuman {
                    name: "bob".to_owned(),
                    friend_ids: vec![],
                })?;

                let alice = conn.create_human(CreateHuman {
                    name: "alice".to_owned(),
                    friend_ids: vec![bob.id],
                })?;

                conn.find_friends_by_human_id(&alice.id)
            })
            .unwrap();

        assert_eq!(friends[0].name, "bob");
    }

    #[test]
    fn update_human_should_return_none() {
        let pool = init_pool();

        let human = pool
            .test_transaction(|conn| {
                conn.update_human(
                    &Uuid::new_v4(),
                    UpdateHuman {
                        name: "no exist".to_owned(),
                        friend_ids: vec![],
                    },
                )
            })
            .unwrap();

        assert_eq!(human, None);
    }

    #[test]
    fn update_human_should_return_some_human() {
        let pool = init_pool();

        let (updated, friends, alice) = pool
            .test_transaction(|conn| {
                let bob = conn.create_human(CreateHuman {
                    name: "bob".to_owned(),
                    friend_ids: vec![],
                })?;
                let alice = conn.create_human(CreateHuman {
                    name: "alice".to_owned(),
                    friend_ids: vec![],
                })?;

                let updated = conn.update_human(
                    &bob.id,
                    UpdateHuman {
                        name: "newname".to_owned(),
                        friend_ids: vec![alice.id],
                    },
                )?;
                let friends = conn.find_friends_by_human_id(&bob.id)?;
                Ok((updated, friends, alice))
            })
            .unwrap();

        assert_matches!(updated, Some(bob) => {
            assert_eq!(bob.name, "newname");
            assert_eq!(friends, vec![alice]);
        })
    }

    #[test]
    fn delete_human_should_return_zero() {
        let pool = init_pool();

        let updated = pool
            .test_transaction(|conn| conn.delete_human(&Uuid::new_v4()))
            .unwrap();

        assert_eq!(updated, 0);
    }

    #[test]
    fn find_friends_by_human_id_should_return_empty() {
        let pool = init_pool();

        let friends = pool
            .test_transaction(|conn| conn.find_friends_by_human_id(&Uuid::new_v4()))
            .unwrap();

        assert_eq!(friends, Vec::<Human>::new());
    }

}
