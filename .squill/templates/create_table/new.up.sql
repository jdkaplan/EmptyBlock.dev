create table TODO_NAME (
    id uuid primary key default gen_random_uuid(),

    -- TODO: Add columns here

    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
);

select manage_updated_at('TODO_NAME');
