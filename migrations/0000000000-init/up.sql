/*
Set up the Squill framework requirements. Naturally, this first migration is
going to break a few rules ;)

Normally, Squill would automatically add a few steps while running a migration.
Most importantly it would...

1. Open a transaction around the migration file. In Postgres, DDL can be done
   in a transaction. This can make some migrations safer, so Squill assumes
   transactions as the default.

2. Call `_squill_claim_migration(id, name)` within that transaction before
   running the file. Since this claim would fail on a duplicate ID, this
   ensures that we never run a migration twice.

It doesn't make sense to call _squill_claim_migration yet, because this is the
migration that defines it!

So this file includes a squill:no-transaction directive (below), which tells
Squill to skip those steps. You can use this in your own migrations if you want
to control the transaction and claim behavior. But remember: if you disable the
automatic transaction, your migration is responsible for recording itself in
the migration log!

You can modify the _squill_claim_migration function if you want to. The only
expectation Squill has of it (besides the signature) is that it writes the
migration ID to the table and fails if that ID is already recorded.

You can also modify (or remove) _squill_require_migration; Squill doesn't use
it. It's useful to call within a migration that would only make sense to run
after some earlier one has completed.

You can also modify the schema_migrations table, but (at least for now) Squill
assumes that the migration log has exactly that name and at least the columns
defined here.
*/
--squill:no-transaction
begin;

create table schema_migrations (
    id bigint primary key,
    name text not null,
    run_at timestamp not null default current_timestamp
);

-- _squill_claim_migration registers a migration in the schema_migrations
-- table. It will fail if the migration ID has already been claimed.
--
-- Squill will call this at the start of every "up" migration transaction. For
-- migrations that cannot be run within transactions, it is the migration's
-- responsibility to call this.
create function _squill_claim_migration(mid bigint, mname text) returns void as $$
    insert into schema_migrations (id, name) values (mid, mname);
$$ language sql;

-- _squill_unclaim_migration removes a migration from the schema_migrations
-- table.
--
-- When iterating on a migration in development, it's useful to have a down
-- migration to reset back to the previous schema. Squill will call this at the
-- start of every "down" migration transaction so the "up" migration can run
-- again. For migrations that cannot be run within transactions, it is the
-- migration's responsibility to call this.
create function _squill_unclaim_migration(mid bigint) returns void as $$ delete
from schema_migrations where id = mid; $$ language sql;

-- _squill_require_migration asserts that the migration ID has already been
-- claimed in the schema_migrations table.
--
-- Call this from within a migration to ensure that another migration has
-- already run to completion.
create function _squill_require_migration(mid bigint) returns void as $$
declare
    mrow schema_migrations%rowtype;
begin
    select * into mrow from schema_migrations where id = mid;
    if not found then
        raise exception 'Required migration has not been run: %', mid;
    end if;
end;
$$ language plpgsql;

-- Normally, this would be the first thing in the migration, but we had to
-- create the schema_migrations table first!
select _squill_claim_migration(0, 'init');

commit;
