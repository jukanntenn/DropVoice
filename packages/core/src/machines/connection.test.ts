import { describe, it, expect } from 'vitest';

import {
  connectionReducer,
  MAX_RETRIES,
  RECONNECT_BASE_DELAY_MS,
  RECONNECT_MAX_DELAY_MS,
  nextReconnectDelay,
} from './connection';
import type { ConnectionState } from './connection';

describe('connectionReducer (§9.2 transition table)', () => {
  // ── CONNECT ───────────────────────────────────────────────────────────────
  describe('CONNECT', () => {
    it('idle → connecting{retries:0}（新发起，重置）', () => {
      expect(connectionReducer({ status: 'idle' }, { type: 'CONNECT' })).toEqual({
        status: 'connecting',
        retries: 0,
      });
    });

    it('offline → connecting{retries:0}（用户手动重试 = 新发起）', () => {
      expect(
        connectionReducer({ status: 'offline', reason: 'exhausted' }, { type: 'CONNECT' })
      ).toEqual({ status: 'connecting', retries: 0 });
    });

    it('reconnecting{r} → connecting{retries:r}（定时重试，保留 r）', () => {
      const next = connectionReducer({ status: 'reconnecting', retries: 3 }, { type: 'CONNECT' });
      expect(next).toEqual({ status: 'connecting', retries: 3 });
    });

    it('connecting → 不变（no-op，同引用）', () => {
      const state: ConnectionState = { status: 'connecting', retries: 0 };
      expect(connectionReducer(state, { type: 'CONNECT' })).toBe(state);
    });

    it('connected → 不变（no-op，同引用）', () => {
      const state: ConnectionState = { status: 'connected' };
      expect(connectionReducer(state, { type: 'CONNECT' })).toBe(state);
    });
  });

  // ── OPEN ──────────────────────────────────────────────────────────────────
  describe('OPEN', () => {
    it('connecting → connected（retries 隐式归零）', () => {
      expect(connectionReducer({ status: 'connecting', retries: 4 }, { type: 'OPEN' })).toEqual({
        status: 'connected',
      });
    });

    it('reconnecting → connected', () => {
      expect(connectionReducer({ status: 'reconnecting', retries: 2 }, { type: 'OPEN' })).toEqual({
        status: 'connected',
      });
    });

    it('idle → 不变（no-op）', () => {
      const state: ConnectionState = { status: 'idle' };
      expect(connectionReducer(state, { type: 'OPEN' })).toBe(state);
    });

    it('connected → 不变（no-op）', () => {
      const state: ConnectionState = { status: 'connected' };
      expect(connectionReducer(state, { type: 'OPEN' })).toBe(state);
    });

    it('offline → 不变（no-op）', () => {
      const state: ConnectionState = { status: 'offline', reason: 'auth' };
      expect(connectionReducer(state, { type: 'OPEN' })).toBe(state);
    });
  });

  // ── CLOSE ─────────────────────────────────────────────────────────────────
  describe('CLOSE', () => {
    it('connected → reconnecting{retries:0}（首次掉线归零）', () => {
      expect(connectionReducer({ status: 'connected' }, { type: 'CLOSE' })).toEqual({
        status: 'reconnecting',
        retries: 0,
      });
    });

    it('connecting{r} → reconnecting{r+1}（+1）', () => {
      expect(connectionReducer({ status: 'connecting', retries: 0 }, { type: 'CLOSE' })).toEqual({
        status: 'reconnecting',
        retries: 1,
      });
    });

    it('reconnecting{r} → reconnecting{r+1}（防御，+1）', () => {
      expect(connectionReducer({ status: 'reconnecting', retries: 4 }, { type: 'CLOSE' })).toEqual({
        status: 'reconnecting',
        retries: 5,
      });
    });

    it(`connecting{MAX} → offline{exhausted}（耗尽）`, () => {
      expect(
        connectionReducer({ status: 'connecting', retries: MAX_RETRIES }, { type: 'CLOSE' })
      ).toEqual({ status: 'offline', reason: 'exhausted' });
    });

    it(`reconnecting{MAX} → offline{exhausted}（耗尽）`, () => {
      expect(
        connectionReducer({ status: 'reconnecting', retries: MAX_RETRIES }, { type: 'CLOSE' })
      ).toEqual({ status: 'offline', reason: 'exhausted' });
    });

    it('idle → 不变（no-op）', () => {
      const state: ConnectionState = { status: 'idle' };
      expect(connectionReducer(state, { type: 'CLOSE' })).toBe(state);
    });

    it('offline → 不变（no-op）', () => {
      const state: ConnectionState = { status: 'offline', reason: 'timeout' };
      expect(connectionReducer(state, { type: 'CLOSE' })).toBe(state);
    });
  });

  // ── FAIL（§9.5 原文 + §7 验收"503 快速重试"：瞬态失败计入退避，耗尽转 offline{reason}）──
  describe('FAIL', () => {
    it('connecting{r} → reconnecting{r+1}（计入退避，如 §7 的 503 DESKTOP_OFFLINE）', () => {
      expect(
        connectionReducer({ status: 'connecting', retries: 0 }, { type: 'FAIL', reason: 'offline' })
      ).toEqual({ status: 'reconnecting', retries: 1 });
      expect(
        connectionReducer({ status: 'connecting', retries: 3 }, { type: 'FAIL', reason: 'timeout' })
      ).toEqual({ status: 'reconnecting', retries: 4 });
    });

    it('reconnecting{r} → reconnecting{r+1}', () => {
      expect(
        connectionReducer(
          { status: 'reconnecting', retries: 2 },
          { type: 'FAIL', reason: 'offline' }
        )
      ).toEqual({ status: 'reconnecting', retries: 3 });
    });

    it(`connecting{MAX}/reconnecting{MAX} + FAIL → offline{reason}（耗尽，保留原因）`, () => {
      expect(
        connectionReducer(
          { status: 'connecting', retries: MAX_RETRIES },
          { type: 'FAIL', reason: 'offline' }
        )
      ).toEqual({ status: 'offline', reason: 'offline' });
      expect(
        connectionReducer(
          { status: 'reconnecting', retries: MAX_RETRIES },
          { type: 'FAIL', reason: 'auth' }
        )
      ).toEqual({ status: 'offline', reason: 'auth' });
    });

    it('connected + FAIL → reconnecting{0}（首次失败归零，与 CLOSE 对称）', () => {
      expect(
        connectionReducer({ status: 'connected' }, { type: 'FAIL', reason: 'timeout' })
      ).toEqual({ status: 'reconnecting', retries: 0 });
    });

    it('idle → 不变（no-op，同引用）', () => {
      const state: ConnectionState = { status: 'idle' };
      expect(connectionReducer(state, { type: 'FAIL', reason: 'offline' })).toBe(state);
    });

    it('offline → 不变（no-op，同引用）', () => {
      const state: ConnectionState = { status: 'offline', reason: 'exhausted' };
      expect(connectionReducer(state, { type: 'FAIL', reason: 'timeout' })).toBe(state);
    });
  });

  // ── DISCONNECT ────────────────────────────────────────────────────────────
  describe('DISCONNECT', () => {
    it.each([
      ['connecting', { status: 'connecting', retries: 3 } satisfies ConnectionState],
      ['connected', { status: 'connected' } satisfies ConnectionState],
      ['reconnecting', { status: 'reconnecting', retries: 5 } satisfies ConnectionState],
      ['offline', { status: 'offline', reason: 'exhausted' } satisfies ConnectionState],
    ])('从 %s → idle', (_label, state) => {
      expect(connectionReducer(state, { type: 'DISCONNECT' })).toEqual({ status: 'idle' });
    });

    it('idle → 同引用（no-op）', () => {
      const state: ConnectionState = { status: 'idle' };
      expect(connectionReducer(state, { type: 'DISCONNECT' })).toBe(state);
    });
  });

  // ── 关键不变量：退避单调升级、耗尽转 offline、定时重试不清零 ───────────────
  describe('关键不变量', () => {
    it('完整退避链：connected → 重试 0..MAX → offline{exhausted}（单调升级）', () => {
      // connected 掉线 → reconnecting{0}（首次归零）→ 定时重试 CONNECT 保留 r。
      let state: ConnectionState = connectionReducer({ status: 'connected' }, { type: 'CLOSE' });
      expect(state).toEqual({ status: 'reconnecting', retries: 0 });

      // retries 0..MAX 各得一次重试机会（共 MAX+1 次 = 6 次延迟 1/2/4/8/16/30s，
      // 与 §9 验收一致）；CONNECT 保留 r、CLOSE 使 r+1，到 r=MAX 再 CLOSE 才耗尽。
      for (let r = 0; r <= MAX_RETRIES; r++) {
        expect(state).toEqual({ status: 'reconnecting', retries: r });
        state = connectionReducer(state, { type: 'CONNECT' });
        expect(state).toEqual({ status: 'connecting', retries: r });
        if (r < MAX_RETRIES) {
          state = connectionReducer(state, { type: 'CLOSE' });
          expect(state).toEqual({ status: 'reconnecting', retries: r + 1 });
        } else {
          // r=MAX 时 CLOSE → r1=MAX+1 > MAX → offline{exhausted}
          state = connectionReducer(state, { type: 'CLOSE' });
        }
      }
      expect(state).toEqual({ status: 'offline', reason: 'exhausted' });
    });

    it('OPEN 成功后 retries 隐式归零（再掉线从 0 起）', () => {
      let state: ConnectionState = { status: 'reconnecting', retries: 4 };
      state = connectionReducer(state, { type: 'CONNECT' });
      state = connectionReducer(state, { type: 'OPEN' });
      expect(state).toEqual({ status: 'connected' });
      // 再掉线 → reconnecting{0}
      expect(connectionReducer(state, { type: 'CLOSE' })).toEqual({
        status: 'reconnecting',
        retries: 0,
      });
    });

    it('offline 后 CONNECT 重置为 connecting{0}（用户手动重试）', () => {
      const state = connectionReducer(
        { status: 'offline', reason: 'exhausted' },
        { type: 'CONNECT' }
      );
      expect(state).toEqual({ status: 'connecting', retries: 0 });
    });
  });

  // ── 常量 + 退避函数 ───────────────────────────────────────────────────────
  describe('constants & nextReconnectDelay', () => {
    it('MAX_RETRIES = 5，退避基准/上限正确', () => {
      expect(MAX_RETRIES).toBe(5);
      expect(RECONNECT_BASE_DELAY_MS).toBe(1_000);
      expect(RECONNECT_MAX_DELAY_MS).toBe(30_000);
    });

    it('nextReconnectDelay 指数翻倍直到上限', () => {
      expect(nextReconnectDelay(0)).toBe(1_000);
      expect(nextReconnectDelay(1)).toBe(2_000);
      expect(nextReconnectDelay(2)).toBe(4_000);
      expect(nextReconnectDelay(3)).toBe(8_000);
      expect(nextReconnectDelay(4)).toBe(16_000);
      expect(nextReconnectDelay(5)).toBe(30_000); // 32s 被封顶
      expect(nextReconnectDelay(10)).toBe(RECONNECT_MAX_DELAY_MS);
    });
  });
});
