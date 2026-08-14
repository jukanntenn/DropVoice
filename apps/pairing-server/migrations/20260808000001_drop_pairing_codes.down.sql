-- 回滚：重建 pairing_codes 表（仅用于回退到旧设计，正常流程不需要）。
CREATE TABLE IF NOT EXISTS pairing_codes (
    code       TEXT    PRIMARY KEY,
    device_id  TEXT    NOT NULL,
    ip         TEXT    NOT NULL,
    port       INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    used       INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (device_id) REFERENCES devices(id)
);
CREATE INDEX IF NOT EXISTS idx_pairing_codes_expires_at ON pairing_codes(expires_at);
