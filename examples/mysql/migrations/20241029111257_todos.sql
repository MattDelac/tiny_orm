CREATE TABLE IF NOT EXISTS todos (
    id          BIGINT          NOT NULL        PRIMARY KEY AUTO_INCREMENT,
    created_at  TIMESTAMP       NOT NULL        DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP       NOT NULL        DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    description TEXT            NOT NULL,
    done        BOOLEAN         NOT NULL        DEFAULT FALSE
);
