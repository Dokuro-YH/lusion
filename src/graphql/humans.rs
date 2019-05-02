use uuid::Uuid;

use crate::db::{
    humans::{CreateHuman, Human, HumanRepository, UpdateHuman},
    PgConn,
};
use crate::error::{self, Result};

pub struct QueryHuman;
pub struct MutationHuman;

graphql_object!(Human: PgConn |&self| {
    field id() -> &Uuid {
        &self.id
    }

    field name() -> &str {
        self.name.as_str()
    }

    field friends(&executor) -> Result<Vec<Human>> {
        let conn = executor.context();

        let friends = conn.find_friends_by_human_id(&self.id)?;
        Ok(friends)
    }
});

graphql_object!(QueryHuman: PgConn |&self| {
    field get(&executor, human_id: Uuid) -> Result<Human> {
        let conn = executor.context();
        let human = conn.find_human(&human_id)?;
        human.ok_or(error::user_error("Not Found"))
    }

    field query(&executor) -> Result<Vec<Human>> {
        let conn = executor.context();
        let humans = conn.find_humans()?;
        Ok(humans)
    }
});

graphql_object!(MutationHuman: PgConn |&self| {
    field create(&executor, input: CreateHuman) -> Result<Human, > {
        let conn = executor.context();
        let human = conn.create_human(input)?;
        Ok(human)
    }

    field update(&executor, human_id: Uuid, input: UpdateHuman) -> Result<Human> {
        let conn = executor.context();
        let human = conn.update_human(&human_id, input)?;
        human.ok_or(error::user_error("Not Found"))
    }

    field delete(&executor, human_id: Uuid) -> Result<()> {
        let conn = executor.context();
        conn.delete_human(&human_id)?;
        Ok(())
    }
});
