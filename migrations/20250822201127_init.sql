CREATE TABLE IF NOT EXISTS auto_voice_channels_installations (
  channel_id INTEGER NOT NULL PRIMARY KEY,
  category_id INTEGER NOT NULL UNIQUE,
  guild_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  created_by INTEGER NOT NULL
);