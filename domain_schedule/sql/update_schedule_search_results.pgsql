INSERT INTO schedule_search_results(remote_id, name, description, type) 
VALUES $1
ON CONFLICT (name) DO UPDATE
SET remote_id = excluded.remote_id, 
    description = excluded.description, 
    type = excluded.type;
