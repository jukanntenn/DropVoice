import { describe, expect, it } from 'vitest';

import { cn } from './cn';

describe('cn', () => {
  it('joins multiple class names', () => {
    expect(cn('a', 'b')).toBe('a b');
  });

  it('skips falsy and conditional values', () => {
    const enabled = false;
    expect(cn('a', enabled && 'b', undefined, null, 'c')).toBe('a c');
  });

  it('dedupes conflicting tailwind utilities (last wins)', () => {
    expect(cn('p-2', 'p-4')).toBe('p-4');
  });

  it('merges arrays and objects', () => {
    expect(cn(['a', 'b'], { c: true, d: false })).toBe('a b c');
  });
});
