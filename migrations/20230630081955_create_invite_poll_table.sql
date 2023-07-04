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


CREATE TYPE invite_poll_status AS ENUM ('open', 'closed');

CREATE TABLE invite_poll (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    status invite_poll_status NOT NULL DEFAULT 'open',
    ends_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER invite_poll_update_updated_at
BEFORE UPDATE ON invite_poll
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
