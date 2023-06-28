pub use sea_orm_migration::prelude::*;

mod m20230628_082939_create_invite_poll_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20230628_082939_create_invite_poll_table::Migration,
        )]
    }
}
