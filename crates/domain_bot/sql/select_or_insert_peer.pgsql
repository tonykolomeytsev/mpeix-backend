WITH new_peer AS (
    INSERT INTO peer(selected_schedule, selecting_schedule)
    SELECT NULL, FALSE
    WHERE NOT EXISTS (
        SELECT native_id FROM peer_by_platform 
        WHERE {platform}_id={id}
    )
    RETURNING *
),
new_peer_by_platform AS (
    INSERT INTO peer_by_platform (native_id, {platform}_id)
    SELECT (SELECT id FROM new_peer), {id}
    WHERE EXISTS (SELECT id FROM new_peer)
)
SELECT * FROM new_peer
UNION
SELECT * FROM peer
WHERE id in (
    SELECT native_id FROM peer_by_platform 
    WHERE {platform}_id={id}
);
