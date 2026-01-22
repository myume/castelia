CREATE TABLE users(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL,
    email VARCHAR(320) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    stream_key_hash BYTEA NOT NULL,
    stream_key BYTEA NOT NULL,
    nonce BYTEA NOT NULL
);
