-- vim: ft=pgsql

CREATE VIEW invite_poll_with_vote_count AS
SELECT
    ip.*,
    COUNT(ipvs.user_id) FILTER (WHERE ipvs.vote = 'yes') AS yes_count,
    COUNT(ipvs.user_id) FILTER (WHERE ipvs.vote = 'maybe') AS maybe_count,
    COUNT(ipvs.user_id) FILTER (WHERE ipvs.vote = 'no') AS no_count
FROM invite_poll AS ip
LEFT JOIN invite_poll_vote_submission AS ipvs ON ipvs.invite_poll_id = ip.id
GROUP BY ip.id;
