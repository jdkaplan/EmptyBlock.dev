create table jobs (
    id uuid primary key default gen_random_uuid(),

    params jsonb not null,
    error text null,
    retries_remaining integer null,

    scheduled_at timestamp not null default now(),
    started_at   timestamp null,
    finished_at  timestamp null,

    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
);

create index jobs_scheduled_at
    on jobs (scheduled_at)
    where finished_at is null;

select manage_updated_at('jobs');
