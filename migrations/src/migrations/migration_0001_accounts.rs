use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new(std::file!())
        .with_up(
            r#"
        CREATE TABLE accounts (
            id BIGSERIAL PRIMARY KEY,
            itchio_user_id BIGINT NOT NULL UNIQUE,
            username TEXT NOT NULL UNIQUE,
            itchio_token TEXT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
        "#,
        )
        .with_down(
            r#"
        DROP TABLE IF EXISTS accounts
        "#,
        )
        .with_up(
            r#"
        CREATE FUNCTION account_lookup(itchio_user_id_in BIGINT, username_in TEXT) RETURNS BIGINT AS $$ 
            DECLARE
                new_account_id BIGINT NOT NULL := 0;
            BEGIN
                INSERT INTO accounts (itchio_user_id, username) VALUES (itchio_user_id_in, username_in) 
                    ON CONFLICT (itchio_user_id) DO UPDATE SET username = username_in
                    RETURNING id INTO new_account_id;
                RETURN new_account_id;
            END;
            $$ LANGUAGE plpgsql;
        "#,
        )
        .with_down(
            r#"
        DROP FUNCTION IF EXISTS account_lookup
        "#,
        )
        .with_up(
            r#"
        CREATE FUNCTION account_get_itchio_token(id_in BIGINT) RETURNS TEXT AS $$ 
            DECLARE
                token TEXT;
            BEGIN
                token := (SELECT itchio_token FROM accounts WHERE id = id_in);
                RETURN token;
            END;
            $$ LANGUAGE plpgsql;
        "#,
        )
        .with_down(
            r#"
        DROP FUNCTION IF EXISTS account_get_itchio_token
        "#,)
        .with_up(
            r#"
        CREATE TABLE installations (
            id UUID PRIMARY KEY,
            account_id BIGINT NULL REFERENCES accounts(id)
        )
        "#,
        )
        .with_down(
            r#"
        DROP TABLE IF EXISTS installations
        "#,
        )
        .with_up(
            r#"
        CREATE FUNCTION installation_lookup(id_in UUID) RETURNS SETOF installations AS $$ 
            BEGIN
                INSERT INTO installations (id) VALUES (id_in) 
                    ON CONFLICT DO NOTHING;
                RETURN QUERY SELECT * FROM installations WHERE id = id_in;
            END;
            $$ LANGUAGE plpgsql;
        "#,
        )
        .with_down(
            r#"
        DROP FUNCTION IF EXISTS installation_lookup
        "#,
        )
        .with_up(
            r#"
        CREATE FUNCTION installation_login(installation_id UUID, account_id_in BIGINT, itchio_token_in TEXT) RETURNS bigint AS $$ 
            DECLARE
                affected_rows bigint;
            BEGIN
                UPDATE accounts SET itchio_token = itchio_token_in WHERE accounts.id = account_id_in;
                UPDATE installations SET account_id = account_id_in WHERE id = installation_id;
                GET DIAGNOSTICS affected_rows = ROW_COUNT;
                PERFORM pg_notify('installation_login', installation_id::text);
                RETURN affected_rows;
            END;
            $$ LANGUAGE plpgsql;
        "#,
        )
        .with_down(
            r#"
        DROP FUNCTION IF EXISTS installation_login
        "#,
        )
        .with_up(
            r#"
        CREATE FUNCTION installation_profile(installation_id UUID) RETURNS TABLE (id BIGINT, username TEXT) AS $$ 
            SELECT accounts.id, accounts.username FROM accounts INNER JOIN installations ON installations.account_id = accounts.id WHERE installations.id = installation_id;
            $$ LANGUAGE sql;
        "#,
        )
        .with_down(
            r#"
        DROP FUNCTION IF EXISTS installation_profile
        "#)
}
