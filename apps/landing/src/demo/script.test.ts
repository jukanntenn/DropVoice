import { describe, expect, it } from 'vitest';

import { cycleLength, DEMO_TIMINGS, demoFrame, demoStillFrame } from './script';

const phrases = ['ab', 'cdef'] as const;

describe('demoFrame', () => {
  it('starts listening with the first character visible', () => {
    const frame = demoFrame(0, phrases);
    expect(frame).toEqual({ phrase: 'ab', phase: 'listening', phoneChars: 1, pcChars: 0 });
  });

  it('reveals phone text progressively during listening', () => {
    const half = demoFrame(DEMO_TIMINGS.listen / 2, phrases);
    expect(half.phase).toBe('listening');
    expect(half.phoneChars).toBe(1); // 2 字符的 50% 四舍五入为 1
    const done = demoFrame(DEMO_TIMINGS.listen - 1, phrases);
    expect(done.phoneChars).toBe(2);
  });

  it('sends with full phone text and empty pc text', () => {
    const frame = demoFrame(DEMO_TIMINGS.listen + 10, phrases);
    expect(frame.phase).toBe('sending');
    expect(frame.phoneChars).toBe(2);
    expect(frame.pcChars).toBe(0);
  });

  it('types onto the pc during typing phase', () => {
    const start = demoFrame(DEMO_TIMINGS.listen + DEMO_TIMINGS.send, phrases);
    expect(start.phase).toBe('typing');
    expect(start.pcChars).toBe(1);
    const end = demoFrame(DEMO_TIMINGS.listen + DEMO_TIMINGS.send + DEMO_TIMINGS.type - 1, phrases);
    expect(end.pcChars).toBe(2);
  });

  it('rests with both surfaces showing the full phrase', () => {
    const frame = demoFrame(cycleLength() - 1, phrases);
    expect(frame.phase).toBe('resting');
    expect(frame.phoneChars).toBe(2);
    expect(frame.pcChars).toBe(2);
  });

  it('rotates to the next phrase after each cycle', () => {
    const second = demoFrame(cycleLength() + 1, phrases);
    expect(second.phrase).toBe('cdef');
    const third = demoFrame(cycleLength() * 2 + 1, phrases);
    expect(third.phrase).toBe('ab'); // 轮换回第一句
  });
});

describe('demoStillFrame', () => {
  it('shows the full phrase on both surfaces', () => {
    expect(demoStillFrame(phrases)).toEqual({
      phrase: 'ab',
      phase: 'resting',
      phoneChars: 2,
      pcChars: 2,
    });
  });
});
