use sqlx_simple_migrator::Migration;

pub fn migration() -> Migration {
    Migration::new("0006")
        // icu collation that is case insensitive that ignores accents
        // Sadly, it appears that unique indexes do not work based collation
        // .with_up("CREATE COLLATION IF NOT EXISTS ignore_case_and_accents (provider = icu, locale = 'und-u-ks-level1-kc-false', deterministic = false);")
        // .with_down("DROP COLLATION IF EXISTS ignore_case_and_accents")
        .with_up("ALTER TABLE accounts DROP COLUMN IF EXISTS screenname")
        .with_up("ALTER TABLE accounts ADD COLUMN login TEXT UNIQUE")
        .with_up("ALTER TABLE accounts ADD COLUMN display_name TEXT")
        .with_down("ALTER TABLE accounts DROP COLUMN IF EXISTS login")
        .with_down("ALTER TABLE accounts DROP COLUMN IF EXISTS display_name")
        .with_down("ALTER TABLE accounts ADD COLUMN IF NOT EXISTS screenname TEXT UNIQUE")
}
