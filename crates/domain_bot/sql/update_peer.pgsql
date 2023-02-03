UPDATE peer
SET 
    selected_schedule={selected_schedule},
    selecting_schedule={selecting_schedule}
WHERE id={id}
RETURNING *;
