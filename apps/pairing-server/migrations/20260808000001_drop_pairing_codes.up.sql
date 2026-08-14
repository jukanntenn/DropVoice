-- WebRTC 信令设计下，配对码改为桌面生成 + 桌面验证（§3.2），
-- 服务器零 code 知识——只是个"SDP 转发板"。删除服务器端 pairing_codes 表。
DROP TABLE IF EXISTS pairing_codes;
