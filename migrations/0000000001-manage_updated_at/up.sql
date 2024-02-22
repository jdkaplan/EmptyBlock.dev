-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- create table users (id serial primary key, updated_at timestamp not null default now());
--
-- select manage_updated_at('users');
-- ```
create function manage_updated_at(_tbl regclass) returns void as $$
begin
    execute format(
        'create trigger set_updated_at before update
         on %s
         for each row execute procedure set_updated_at()',
        _tbl
    );
end;
$$ language plpgsql;

create function set_updated_at() returns trigger as $$
begin
    if (
        new is distinct from old and
        new.updated_at is not distinct from old.updated_at
    ) then
        new.updated_at := current_timestamp;
    end if;
    return new;
end;
$$ language plpgsql;
