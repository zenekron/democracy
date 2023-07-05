-- vim: ft=pgsql

-- Trigger function that updates the `updated_at` attribute of a record to the
-- current time.
CREATE FUNCTION update_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
	BEGIN
		NEW.updated_at = now();
		RETURN NEW;
	END;
$$;


CREATE TYPE invite_poll_outcome AS ENUM ('allow', 'deny');

CREATE TABLE invite_poll (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id varchar NOT NULL,
    user_id varchar NOT NULL,
    channel_id varchar,
    message_id varchar,
    outcome invite_poll_outcome,
    ends_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER invite_poll_update_updated_at
BEFORE UPDATE ON invite_poll
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
