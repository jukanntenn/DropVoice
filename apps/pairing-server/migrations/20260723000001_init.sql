-- 初始化 schema（spec 11 §9.1）。
-- devices 表：设备地址簿（id 由客户端生成，UUID v4）。
CREATE TABLE IF NOT EXISTS devices (
    id            TEXT    PRIMARY KEY,            -- 设备 UUID v4（客户端生成）
    platform      TEXT    NOT NULL,               -- "desktop"
    device_name   TEXT,                            -- 固化的设备名（可空）
    ip            TEXT    NOT NULL,
    port          INTEGER NOT NULL,
    pairing_token TEXT    NOT NULL UNIQUE,         -- 64 字符
    is_online     INTEGER NOT NULL DEFAULT 0,
    last_seen     INTEGER NOT NULL,                -- unix 时间戳（秒）
    created_at    INTEGER NOT NULL                 -- unix 时间戳（秒）
);

CREATE INDEX IF NOT EXISTS idx_devices_pairing_token ON devices(pairing_token);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen     ON devices(last_seen);

-- pairing_codes 表：一次性配对码（6 位数字）。
CREATE TABLE IF NOT EXISTS pairing_codes (
    code       TEXT    PRIMARY KEY,                -- 6 位数字
    device_id  TEXT    NOT NULL,
    ip         TEXT    NOT NULL,                   -- 冗余，查询直接返回
    port       INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,                   -- unix 时间戳（秒）
    used       INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (device_id) REFERENCES devices(id)
);

CREATE INDEX IF NOT EXISTS idx_pairing_codes_expires_at ON pairing_codes(expires_at);
