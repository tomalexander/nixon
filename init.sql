BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS props (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       name TEXT NOT NULL,
       value TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_props_name ON props (name);

INSERT INTO props (name, value) VALUES ('api_key', 'yoUrApIkEyHerE');
INSERT INTO props (name, value) VALUES ('server', 'your.hipchat.server.com');

CREATE TABLE IF NOT EXISTS rooms (
       id INTEGER PRIMARY KEY,
       is_archived BOOLEAN NOT NULL,
       name TEXT NOT NULL,
       privacy TEXT NOT NULL,
       version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
       id TEXT NOT NULL PRIMARY KEY,
       room_id INTEGER NOT NULL,
       color TEXT,
       date INTEGER NOT NULL,
       sender TEXT NOT NULL,
       message TEXT,
       message_format TEXT
);
CREATE INDEX idx_messages_room_id ON messages (room_id);

END TRANSACTION;
