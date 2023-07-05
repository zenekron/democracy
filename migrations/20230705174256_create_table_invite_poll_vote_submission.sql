-- vim: ft=pgsql

CREATE TYPE invite_poll_vote AS ENUM ('yes', 'no');

CREATE TABLE invite_poll_vote_submission (
    invite_poll_id uuid NOT NULL REFERENCES invite_poll (id),
    user_id varchar NOT NULL,
    vote invite_poll_vote NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (invite_poll_id, user_id)
);

CREATE TRIGGER invite_poll_vote_submission_update_updated_at
BEFORE UPDATE ON invite_poll_vote_submission
FOR EACH ROW
EXECUTE FUNCTION update_updated_at();
