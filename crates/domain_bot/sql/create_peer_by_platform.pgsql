CREATE TABLE IF NOT EXISTS peer_by_platform(
  native_id INTEGER REFERENCES peer(id) 
    ON UPDATE CASCADE 
    ON DELETE CASCADE,
  telegram_id BIGINT DEFAULT NULL,
  vk_id BIGINT DEFAULT NULL
);
