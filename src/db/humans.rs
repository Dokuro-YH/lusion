///! Human database reposiroty.
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::PgConn;
use crate::error::{ErrorKind, Result, ResultExt};
use crate::schema::{human_friends, humans};

#[derive(Clone, Queryable)]
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

/// Human reposiroty
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

        Ok(humans.load(&self.conn).context(ErrorKind::DbError)?)
    }

    fn find_human(&self, id: &Uuid) -> Result<Option<Human>> {
        Ok(humans::table
            .find(id)
            .first(&self.conn)
            .optional()
            .context(ErrorKind::DbError)?)
    }

    fn create_human(&self, input: CreateHuman) -> Result<Human> {
        use crate::schema::humans::dsl::*;

        let human_id = Uuid::new_v4();
        let human: Human = diesel::insert_into(humans)
            .values((id.eq(&human_id), name.eq(&input.name)))
            .get_result(&self.conn)
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
            .execute(&self.conn)
            .context(ErrorKind::DbError)?;

        Ok(human)
    }

    fn update_human(&self, human_id: &Uuid, input: UpdateHuman) -> Result<Option<Human>> {
        use crate::schema::humans::dsl::*;

        let human: Option<Human> = diesel::update(humans.find(human_id))
            .set(name.eq(&input.name))
            .get_result(&self.conn)
            .optional()
            .context(ErrorKind::DbError)?;

        match human {
            None => Ok(None),
            Some(human) => {
                let _ = diesel::delete(human_friends::table)
                    .filter(human_friends::human_id.eq(human_id))
                    .execute(&self.conn)
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
                    .execute(&self.conn)
                    .context(ErrorKind::DbError)?;
                Ok(Some(human))
            }
        }
    }

    fn delete_human(&self, human_id: &Uuid) -> Result<usize> {
        use crate::schema::humans::dsl::*;

        let _ = diesel::delete(human_friends::table)
            .filter(human_friends::friend_id.eq(human_id))
            .execute(&self.conn)
            .context(ErrorKind::DbError)?;
        let _ = diesel::delete(human_friends::table)
            .filter(human_friends::human_id.eq(human_id))
            .execute(&self.conn)
            .context(ErrorKind::DbError)?;
        let updated = diesel::delete(humans.find(human_id))
            .execute(&self.conn)
            .context(ErrorKind::DbError)?;

        Ok(updated)
    }

    fn find_friends_by_human_id(&self, human_id: &Uuid) -> Result<Vec<Human>> {
        use diesel::dsl::any;

        let friend_ids = human_friends::table
            .select(human_friends::friend_id)
            .filter(human_friends::human_id.eq(human_id))
            .load::<Uuid>(&self.conn)
            .context(ErrorKind::DbError)?;

        Ok(humans::table
            .filter(humans::id.eq(any(friend_ids)))
            .load(&self.conn)
            .context(ErrorKind::DbError)?)
    }
}
