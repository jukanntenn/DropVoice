/**
 * Hero 活体演示的时间线（纯函数，无副作用）。
 *
 * 一轮循环：listening（手机逐字浮现）→ sending（粒子流向电脑）
 * → typing（电脑逐字上屏）→ resting（静止展示）→ 换下一句。
 * 组件按 elapsedMs 调用 demoFrame 计算渲染状态；
 * prefers-reduced-motion 时组件直接渲染 resting 终态，不走此循环。
 */
export type DemoPhase = 'listening' | 'sending' | 'typing' | 'resting';

export interface DemoTimings {
  /** 手机侧文字逐字浮现。 */
  listen: number;
  /** 粒子从手机卡流向电脑卡。 */
  send: number;
  /** 电脑侧逐字打出。 */
  type: number;
  /** 静止展示，然后进入下一句。 */
  rest: number;
}

export const DEMO_TIMINGS: DemoTimings = {
  listen: 2000,
  send: 600,
  type: 1600,
  rest: 1600,
};

export interface DemoFrame {
  phrase: string;
  phase: DemoPhase;
  /** 手机卡当前显示的字符数。 */
  phoneChars: number;
  /** 电脑卡当前显示的字符数。 */
  pcChars: number;
}

export function cycleLength(timings: DemoTimings = DEMO_TIMINGS): number {
  return timings.listen + timings.send + timings.type + timings.rest;
}

/** 字符数按时间线性推进，首帧即为 1 个字符（空文本没有"在打字"的感觉）。 */
function charsAt(elapsed: number, duration: number, total: number): number {
  if (total <= 0) return 0;
  const ratio = Math.min(Math.max(elapsed / duration, 0), 1);
  return Math.max(1, Math.round(ratio * total));
}

export function demoFrame(
  elapsedMs: number,
  phrases: readonly string[],
  timings?: DemoTimings
): DemoFrame {
  const t = timings ?? DEMO_TIMINGS;
  const cycle = cycleLength(t);
  const total = Math.max(0, elapsedMs);
  const phrase = phrases[Math.floor(total / cycle) % phrases.length] ?? '';
  const local = total % cycle;

  if (local < t.listen) {
    return {
      phrase,
      phase: 'listening',
      phoneChars: charsAt(local, t.listen, phrase.length),
      pcChars: 0,
    };
  }
  if (local < t.listen + t.send) {
    return { phrase, phase: 'sending', phoneChars: phrase.length, pcChars: 0 };
  }
  if (local < t.listen + t.send + t.type) {
    return {
      phrase,
      phase: 'typing',
      phoneChars: phrase.length,
      pcChars: charsAt(local - t.listen - t.send, t.type, phrase.length),
    };
  }
  return { phrase, phase: 'resting', phoneChars: phrase.length, pcChars: phrase.length };
}

/** reduced-motion 的定格终态：两个表面都显示完整句子。 */
export function demoStillFrame(phrases: readonly string[]): DemoFrame {
  const phrase = phrases[0] ?? '';
  return { phrase, phase: 'resting', phoneChars: phrase.length, pcChars: phrase.length };
}
