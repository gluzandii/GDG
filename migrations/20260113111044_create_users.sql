create table users
(
    id            bigserial primary key,
    username      text        not null unique check (char_length(username) <= 16),
    email         text        not null unique,
    password_hash text        not null,
    bio           text        not null default '' check (char_length(bio) <= 160),
    created_at    timestamptz not null default now(),
    updated_at    timestamptz not null default now()
);