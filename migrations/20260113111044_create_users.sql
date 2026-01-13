create table users
(
    id            bigserial primary key,
    username      text        not null unique check (char_length(username) <= 16),
    email         text        not null unique check (email ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$'),
    password_hash text        not null,
    created_at    timestamptz not null default now()
);