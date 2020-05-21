use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(std::file!())
        .with_up(
            r#"
            ALTER TABLE accounts ADD COLUMN map INT NOT NULL DEFAULT 0
        "#,
        )
        .with_down(
            r#"
            ALTER TABLE accounts DROP COLUMN IF EXISTS map 
        "#,
        )
        .with_up(
            r#"
            ALTER TABLE accounts ADD COLUMN last_update_timestamp FLOAT NULL
            "#
        )
        .with_down(
            r#"
            ALTER TABLE accounts DROP COLUMN IF EXISTS last_update_timestamp 
        "#,
        )
        .with_up(
            r#"
            ALTER TABLE accounts ADD COLUMN x_offset FLOAT(24) NOT NULL DEFAULT 0
        "#,
        )
        .with_down(
            r#"
            ALTER TABLE accounts DROP COLUMN IF EXISTS x_offset
        "#,
        )
        .with_up(
            r#"
            ALTER TABLE accounts ADD COLUMN horizontal_input FLOAT(24) NOT NULL DEFAULT 0
        "#,
        )
        .with_down(
            r#"
            ALTER TABLE accounts DROP COLUMN IF EXISTS horizontal_input
        "#,
        )
        .with_up(
            r#"
            DROP FUNCTION installation_profile(UUID);
        "#,
        )
        .with_up(
            r#"
        CREATE OR REPLACE FUNCTION installation_profile(installation_id UUID) RETURNS SETOF accounts AS $$ 
            SELECT accounts.* FROM accounts INNER JOIN installations ON installations.account_id = accounts.id WHERE installations.id = installation_id;
            $$ LANGUAGE sql;
        "#,
        )
        .with_down(
            r#"
        CREATE OR REPLACE FUNCTION installation_profile(installation_id UUID) RETURNS TABLE (id BIGINT, username TEXT) AS $$ 
            SELECT accounts.id, accounts.username FROM accounts INNER JOIN installations ON installations.account_id = accounts.id WHERE installations.id = installation_id;
            $$ LANGUAGE sql;
        "#,
        )
        .with_down(
            r#"
            DROP FUNCTION installation_profile(UUID);
        "#,
        )
        .with_up(
            r#"
        CREATE OR REPLACE FUNCTION account_update_inputs(account_id_in BIGINT, x_offset_in FLOAT(24), update_timestamp FLOAT, horizontal_input_in FLOAT(24)) RETURNS TABLE (id BIGINT) AS $$ 
            UPDATE accounts SET x_offset = x_offset_in, horizontal_input = horizontal_input_in, last_update_timestamp = update_timestamp WHERE id = account_id_in RETURNING id;
            $$ LANGUAGE sql;
        "#,
        )
        .with_down(
            r#"
            DROP FUNCTION IF EXISTS account_update_inputs(BIGINT,FLOAT(24),FLOAT,FLOAT(24));
        "#,
        )
        .with_up(
            r#"
        CREATE OR REPLACE FUNCTION account_update_walk(account_id_in BIGINT, current_timestamp_in FLOAT, last_world_update FLOAT, walk_speed FLOAT(24)) RETURNS TABLE (x_offset FLOAT(24)) AS $$ 
            UPDATE 
                accounts 
            SET 
                x_offset = x_offset + walk_speed * (current_timestamp_in - GREATEST(COALESCE(last_update_timestamp, last_world_update), last_world_update)) * horizontal_input, 
                last_update_timestamp = current_timestamp_in
            WHERE id = account_id_in RETURNING x_offset;
            $$ LANGUAGE sql;
        "#, // TODO make this global, make it not update last_update_timestamp, make it so that last_update_timestamp is basically the last time it's been seen
        )
        .with_down(
            r#"
            DROP FUNCTION IF EXISTS account_update_walk(BIGINT,FLOAT,FLOAT,FLOAT(24));
        "#,
        )
        .with_up(
            r#"
        CREATE OR REPLACE FUNCTION account_list_current(since_timestamp_in FLOAT) RETURNS SETOF accounts AS $$ 
            SELECT * FROM accounts WHERE last_update_timestamp is not null AND last_update_timestamp > since_timestamp_in;
            $$ LANGUAGE sql;
        "#,
        )
        .with_down(
            r#"
            DROP FUNCTION IF EXISTS account_list_current(FLOAT);
        "#,
        )
        .debug()
}
