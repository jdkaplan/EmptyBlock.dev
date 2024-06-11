create table users (
    id uuid primary key default gen_random_uuid(),
    recurse_user_id bigint not null,

    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp,

    unique (recurse_user_id)
);

select manage_updated_at('users');
