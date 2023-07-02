-- vim: ft=pgsql

CREATE VIEW invite_poll_vote_count AS
SELECT
    invite_poll_id,
    COUNT(user_id) FILTER (WHERE vote = 'yes') AS yes_count,
    COUNT(user_id) FILTER (WHERE vote = 'maybe') AS maybe_count,
    COUNT(user_id) FILTER (WHERE vote = 'no') AS no_count
FROM invite_poll_vote_submission
GROUP BY invite_poll_id;
