BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS props (
       id INTEGER PRIMARY KEY AUTOINCREMENT,
       name TEXT NOT NULL,
       value TEXT NOT NULL
);

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
       message_id INTEGER PRIMARY KEY AUTOINCREMENT, -- Private to DB
       room_id INTEGER NOT NULL,
       id TEXT NOT NULL,
       color TEXT,
       date INTEGER NOT NULL,
       sender TEXT NOT NULL,
       message TEXT,
       message_format TEXT
);

END TRANSACTION;
