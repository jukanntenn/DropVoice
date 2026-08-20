import type { ReactNode } from 'react';

/** 章节编号 kicker：mono 小字，编号 + 标题（如 "02 · 隐私"）。 */
export function SectionKicker({ children }: { children: ReactNode }) {
  return (
    <p className="text-primary-600 dark:text-primary-400 font-mono text-xs font-medium tracking-[0.2em]">
      {children}
    </p>
  );
}
