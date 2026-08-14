/**
 * 连接状态机（§9.2 重写：纯 reducer，无副作用）。
 *
 * 设计目标：消除旧机的"永久 connecting"。retries 单调递增，耗尽转 `offline`
 * （可见、可操作）。瞬态原因用 `OfflineReason` 表达。
 *
 * 状态机描述**单台设备**的连接生命周期；手机 hook 维护
 * `Record<deviceId, ConnectionState>` 并按设备派发事件。reducer 是纯函数
 * （spec 02 §5 约束 3）——不碰定时器/socket/任何副作用 API。副作用（transport
 * 创建、重连定时器）由 `apps/mobile/src/hooks/useConnections.ts` 拥有。
 *
 * **不变量**（实施权威，见 §9.2 转移表 + §9.5）：
 * - `retries` 在 `CLOSE`/`FAIL`（来自 `connecting`/`reconnecting`）时 +1；首次掉线
 *   （`connected`→）置 0；`OPEN`→`connected` 隐式归零。
 * - 定时重试路径 `reconnecting → connecting`（`CONNECT`）**不清零** retries。
 * - 退避单调升级 → 耗尽转 `offline`（CLOSE → `exhausted`；FAIL → 保留原因）。
 * - 无效转移返回**同一引用**（`return state;`），调用方靠 `===` 检测 no-op。
 *
 * **FAIL 语义（§9.5 原文 + §7 验收"503 快速重试"）**：瞬态失败（DESKTOP_OFFLINE/
 * CONNECTION_TIMEOUT/RATE_LIMITED/DEVICE_NOT_FOUND）**计入退避**，耗尽转
 * `offline{reason}`。曾实现过"FAIL → 立即 offline"（方案 A），实测让桌面 SSE
 * 重建窗口内的 503 直接把手机停车在 offline（不再自动重试，连接回归），
 * 与 §7 验收"快速重试"矛盾，故回归规范原文。transport 失败路径 error 后紧随
 * close 的双重计数由 hook 侧提前 detach transport（stale 守卫）解决。
 */

/** 离线原因（§9.2）。 */
export type OfflineReason =
  'exhausted' | 'refused' | 'unreachable' | 'timeout' | 'auth' | 'offline' | 'unknown';

export type ConnectionState =
  | { status: 'idle' }
  | { status: 'connecting'; retries: number }
  | { status: 'connected' }
  | { status: 'reconnecting'; retries: number }
  | { status: 'offline'; reason: OfflineReason };

export type ConnectionEvent =
  // 新发起（idle/offline）或定时重试（reconnecting）；reducer 按来源态区分
  // retries 是否重置（idle/offline→0，reconnecting→保留 r）。
  | { type: 'CONNECT' }
  // DataChannel open（connecting/reconnecting → connected）。
  | { type: 'OPEN' }
  // transport/channel 关闭（计入退避；耗尽转 offline{exhausted}）。
  | { type: 'CLOSE' }
  // 显式失败（DESKTOP_OFFLINE / 凭据失效回退穷尽）→ 立即 offline{reason}。
  | { type: 'FAIL'; reason: OfflineReason }
  // 用户主动断开 → idle。
  | { type: 'DISCONNECT' };

/** 重连尝试上限（§9.2）。 */
export const MAX_RETRIES = 5;
/** 指数退避基准（§9.2）。 */
export const RECONNECT_BASE_DELAY_MS = 1_000;
/** 指数退避上限（§9.2）。 */
export const RECONNECT_MAX_DELAY_MS = 30_000;

/**
 * 指数退避（带上限）。导出供 hook 在 reducer 决定的 delay 上调度定时器，
 * 而 reducer 本身不调用 `setTimeout`。
 */
export function nextReconnectDelay(retries: number): number {
  return Math.min(RECONNECT_BASE_DELAY_MS * 2 ** retries, RECONNECT_MAX_DELAY_MS);
}

/**
 * 纯 reducer，实现 §9.2 转移表（唯一权威）。
 *
 * 无效转移返回前一状态（同一引用），调用方可靠引用相等检测 no-op。无副作用。
 */
export function connectionReducer(state: ConnectionState, event: ConnectionEvent): ConnectionState {
  switch (event.type) {
    case 'CONNECT': {
      // idle/offline → 新发起（重置 retries=0）。
      if (state.status === 'idle' || state.status === 'offline') {
        return { status: 'connecting', retries: 0 };
      }
      // reconnecting → 定时重试（保留 retries）。
      if (state.status === 'reconnecting') {
        return { status: 'connecting', retries: state.retries };
      }
      // connecting/connected → 不变。
      return state;
    }

    case 'OPEN': {
      if (state.status === 'connecting' || state.status === 'reconnecting') {
        return { status: 'connected' };
      }
      return state;
    }

    case 'CLOSE': {
      // 已连接掉线 → 首次重连（retries 归零）。
      if (state.status === 'connected') {
        return { status: 'reconnecting', retries: 0 };
      }
      // 连接/重连中关闭 → retries+1，耗尽转 offline{exhausted}。
      if (state.status === 'connecting' || state.status === 'reconnecting') {
        const next = state.retries + 1;
        if (next > MAX_RETRIES) {
          return { status: 'offline', reason: 'exhausted' };
        }
        return { status: 'reconnecting', retries: next };
      }
      // idle/offline → 不变。
      return state;
    }

    case 'FAIL': {
      // §9.5（实施权威）+ §7 验收（"503 快速重试"）：瞬态失败计入退避，
      // 耗尽转 offline{reason}（保留原因）。曾按"方案 A"实现过"立即 offline"，
      // 实测导致桌面 SSE 重建窗口内的 503 让手机永久停车（回归），故回归规范原文。
      if (state.status === 'connected') {
        return { status: 'reconnecting', retries: 0 };
      }
      if (state.status === 'connecting' || state.status === 'reconnecting') {
        const next = state.retries + 1;
        if (next > MAX_RETRIES) {
          return { status: 'offline', reason: event.reason };
        }
        return { status: 'reconnecting', retries: next };
      }
      return state;
    }

    case 'DISCONNECT': {
      // 主动断开 → idle（已是 idle 则保持同一引用，no-op）。
      return state.status === 'idle' ? state : { status: 'idle' };
    }

    default: {
      const _exhaustive: never = event;
      void _exhaustive;
      return state;
    }
  }
}
