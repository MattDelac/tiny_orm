CREATE TABLE IF NOT EXISTS todo (
    id          INTEGER PRIMARY KEY NOT NULL,
    created_at  Text                NOT NULL,
    updated_at  Text                NOT NULL,
    description TEXT                NOT NULL,
    done        BOOLEAN             NOT NULL DEFAULT 0
);
