CREATE TABLE invite (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id bigint NOT NULL,
    user_id bigint NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE FUNCTION invite_update_timestamps() RETURNS trigger AS $on_update$
	BEGIN
		NEW.updated_at = now();
		RETURN NEW;
	END;
$on_update$ LANGUAGE plpgsql;

CREATE TRIGGER invite_on_update
BEFORE UPDATE ON invite
FOR EACH ROW
EXECUTE FUNCTION invite_update_timestamps();
