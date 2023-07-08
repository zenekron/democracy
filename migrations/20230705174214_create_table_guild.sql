-- vim: ft=pgsql

CREATE TABLE guild (
    id varchar PRIMARY KEY, -- GuildId
    invite_channel_id varchar NOT NULL, -- ChannelId
    invite_poll_quorum real NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT guild_invite_poll_quorum_is_percentage CHECK (
        invite_poll_quorum >= 0 AND invite_poll_quorum <= 1
    )
);

CREATE TRIGGER guild_update_updated_at
BEFORE UPDATE ON guild
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
