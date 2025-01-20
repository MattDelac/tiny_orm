DROP TABLE IF EXISTS todo;
CREATE TABLE IF NOT EXISTS todo (
    id          uuid PRIMARY KEY             NOT NULL DEFAULT gen_random_uuid(),
    created_at  timestamp with time zone     NOT NULL DEFAULT (now() at time zone 'utc'),
    updated_at  timestamp with time zone     NOT NULL DEFAULT (now() at time zone 'utc'),
    deleted_at  timestamp with time zone     NULL,
    description TEXT                         NOT NULL,
    done        BOOLEAN                      NOT NULL DEFAULT FALSE
);
