CREATE TYPE stream_status AS ENUM(
    'offline',
    'unpublished',
    'published'
);

CREATE TABLE IF NOT EXISTS broadcasts(
    id SERIAL PRIMARY KEY,
    channel_name TEXT UNIQUE NOT NULL,
    title TEXT DEFAULT '' NOT NULL,
    start_time TIMESTAMPTZ,
    status stream_status DEFAULT 'offline' NOT NULL,
    private BOOLEAN DEFAULT false NOT NULL
);

