create table users
(
    id            bigserial primary key,
    username      text        not null unique check (char_length(username) <= 16),
    email         text        not null unique,
    password_hash text        not null,
    created_at    timestamptz not null default now()
);