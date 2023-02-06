CREATE TABLE IF NOT EXISTS peer(
  id BIGSERIAL PRIMARY KEY,
  selected_schedule VARCHAR DEFAULT '' NOT NULL,
  selected_schedule_type VARCHAR DEFAULT '' NOT NULL,
  selecting_schedule BOOLEAN DEFAULT FALSE NOT NULL
);
