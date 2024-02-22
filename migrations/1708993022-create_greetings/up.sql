create table greetings (
    id uuid primary key default gen_random_uuid(),

    greeting text not null check (greeting != ''),

    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),

    unique (greeting)
);

select manage_updated_at('greetings');
