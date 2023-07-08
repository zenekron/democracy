-- vim: ft=pgsql

CREATE TYPE invite_poll_outcome AS ENUM ('allow', 'deny');

CREATE TABLE invite_poll (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(), -- InvitePollId
    guild_id varchar NOT NULL REFERENCES guild (id), -- GuildId
    user_id varchar NOT NULL, -- UserId
    channel_id varchar, -- ChannelId
    message_id varchar, -- MessageId
    outcome invite_poll_outcome,
    ends_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER invite_poll_update_updated_at
BEFORE UPDATE ON invite_poll
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
