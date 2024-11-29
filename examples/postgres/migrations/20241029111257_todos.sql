CREATE TABLE IF NOT EXISTS todos (
    id          uuid PRIMARY KEY             NOT NULL DEFAULT gen_random_uuid(),
    created_at  timestamp with time zone     NOT NULL DEFAULT (now() at time zone 'utc'),
    updated_at  timestamp with time zone     NOT NULL DEFAULT (now() at time zone 'utc'),
    description TEXT                         NOT NULL,
    done        BOOLEAN                      NOT NULL DEFAULT FALSE
);
