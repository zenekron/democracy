CREATE TYPE invite_poll_vote AS ENUM ('yes', 'maybe', 'no');


CREATE TABLE invite_poll_vote_submission (
    invite_poll_id uuid NOT NULL REFERENCES invite_poll (id),
    user_id bigint NOT NULL,
    vote invite_poll_vote NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (invite_poll_id, user_id)
);

CREATE TRIGGER invite_poll_vote_submission_update_updated_at
BEFORE UPDATE ON invite_poll_vote_submission
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();


ALTER TABLE invite_poll
ADD COLUMN yes_count int NOT NULL DEFAULT 0,
ADD COLUMN maybe_count int NOT NULL DEFAULT 0,
ADD COLUMN no_count int NOT NULL DEFAULT 0;

CREATE FUNCTION invite_poll_update_counters() RETURNS trigger AS $BODY$
	DECLARE
		col_name text;
		statement text;

	BEGIN
		-- increment new counter
		col_name := NEW.vote || '_count';
		statement := 'UPDATE invite_poll SET ' || col_name || ' = ' || col_name || ' + 1';

		-- decrement new counter
		IF (TG_OP = 'UPDATE') THEN
			col_name := OLD.vote || '_count';
			statement := statement || ', ' || col_name || ' = ' || col_name || ' - 1';
		END IF;

		RAISE NOTICE 'invite_poll_update_counters: %', statement;
		EXECUTE statement;

		RETURN NEW;
	END;
$BODY$ LANGUAGE plpgsql;

CREATE TRIGGER invite_poll_vote_submission_update_counters_insert
AFTER INSERT ON invite_poll_vote_submission
FOR EACH ROW
EXECUTE FUNCTION invite_poll_update_counters();

CREATE TRIGGER invite_poll_vote_submission_update_counters_update
AFTER UPDATE ON invite_poll_vote_submission
FOR EACH ROW
WHEN (old.vote != new.vote)
EXECUTE FUNCTION invite_poll_update_counters();
