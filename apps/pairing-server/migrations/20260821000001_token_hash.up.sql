-- 凭据模型重设计（token 不再明文落库）。SQLite 无法 DROP 带 UNIQUE 约束的列，
-- 采用表重建：
-- - pairing_token（明文，UNIQUE）→ token_hash（SHA-256 hex）
-- - 新增 token_issued_at：token 轮换的时间依据（旧实现误用 created_at，
--   语义漂移一并修正）
-- 存量明文 token 不做迁移（系统未上线，仅 dev/staging 存量）：token_hash 置空
-- 视为失效，持旧 token 的桌面端认证失败 → 裸重注册被 401 → 客户端自动重置
-- 设备身份（一次性成本）。
CREATE TABLE devices_new (
    id              TEXT    PRIMARY KEY,            -- 设备 UUID v4（客户端生成）
    platform        TEXT    NOT NULL,               -- "desktop"
    device_name     TEXT,                           -- 固化的设备名（可空）
    ip              TEXT    NOT NULL,
    port            INTEGER NOT NULL,
    token_hash      TEXT    NOT NULL DEFAULT '',    -- SHA-256 hex；空 = 迁移存量失效
    token_issued_at INTEGER NOT NULL DEFAULT 0,     -- unix 秒（轮换依据）
    is_online       INTEGER NOT NULL DEFAULT 0,
    last_seen       INTEGER NOT NULL,               -- unix 秒
    created_at      INTEGER NOT NULL                -- unix 秒
);

INSERT INTO devices_new (id, platform, device_name, ip, port,
                         token_hash, token_issued_at, is_online, last_seen, created_at)
SELECT id, platform, device_name, ip, port,
       '', created_at, is_online, last_seen, created_at
FROM devices;

DROP TABLE devices;
ALTER TABLE devices_new RENAME TO devices;

CREATE INDEX IF NOT EXISTS idx_devices_token_hash ON devices(token_hash);
CREATE INDEX IF NOT EXISTS idx_devices_last_seen  ON devices(last_seen);
