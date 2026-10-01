-- 逆迁移：恢复明文 token 列（hash 无法还原为明文，恢复为空串占位）。
CREATE TABLE devices_old (
    id            TEXT    PRIMARY KEY,
    platform      TEXT    NOT NULL,
    device_name   TEXT,
    ip            TEXT    NOT NULL,
    port          INTEGER NOT NULL,
    pairing_token TEXT    NOT NULL DEFAULT '',
    is_online     INTEGER NOT NULL DEFAULT 0,
    last_seen     INTEGER NOT NULL,
    created_at    INTEGER NOT NULL
);

INSERT INTO devices_old (id, platform, device_name, ip, port,
                         pairing_token, is_online, last_seen, created_at)
SELECT id, platform, device_name, ip, port,
       '', is_online, last_seen, created_at
FROM devices;

DROP TABLE devices;
ALTER TABLE devices_old RENAME TO devices;

CREATE INDEX IF NOT EXISTS idx_devices_pairing_token ON devices(pairing_token);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen     ON devices(last_seen);
