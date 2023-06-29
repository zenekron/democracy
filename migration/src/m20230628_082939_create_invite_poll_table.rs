use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InvitePoll::Table)
                    .col(ColumnDef::new(InvitePoll::Id).uuid().primary_key())
                    .col(
                        ColumnDef::new(InvitePoll::GuildId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InvitePoll::UserId).big_unsigned().not_null())
                    .col(ColumnDef::new(InvitePoll::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(InvitePoll::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InvitePoll::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum InvitePoll {
    Table,
    Id,
    GuildId,
    UserId,
    CreatedAt,
    UpdatedAt,
}
