UPDATE peer
SET 
    selected_schedule='{selected_schedule}',
    selected_schedule_type='{selected_schedule_type}',
    selecting_schedule={selecting_schedule}
WHERE id={id}
RETURNING *;
