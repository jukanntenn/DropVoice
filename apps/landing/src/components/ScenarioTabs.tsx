import { useEffect, useState } from 'react';
import { useReducedMotion } from 'framer-motion';

import { cn } from '@dropvoice/ui';

/**
 * 三步原理 · 步骤 3 的场景切换器：聊天 / 文档 / 终端三个接收端，
 * 切换时逐字打出对应示例——把「它就是你的键盘」一次讲透。
 */
export interface Scenario {
  id: string;
  label: string;
  text: string;
}

export function ScenarioTabs({ scenarios }: { scenarios: Scenario[] }) {
  const [activeId, setActiveId] = useState(scenarios[0]?.id ?? '');
  const active = scenarios.find((s) => s.id === activeId) ?? scenarios[0];
  const { shown, typing } = useTypewriter(active?.text ?? '');

  return (
    <div className="flex min-h-36 flex-col">
      <div className="flex gap-1.5" role="tablist" aria-label="Scenarios">
        {scenarios.map((s) => (
          <button
            key={s.id}
            type="button"
            role="tab"
            aria-selected={s.id === activeId}
            onClick={() => setActiveId(s.id)}
            className={cn(
              'rounded-full border px-2.5 py-1 text-xs transition-all active:scale-95',
              s.id === activeId
                ? 'border-primary-500/40 bg-primary/10 text-primary-700 dark:text-primary-300 font-medium'
                : 'border-white/50 bg-white/40 text-neutral-600 hover:bg-white/60 dark:border-white/10 dark:bg-slate-800/40 dark:text-neutral-300 dark:hover:bg-slate-800/60'
            )}
          >
            {s.label}
          </button>
        ))}
      </div>

      <div className="mt-3 flex flex-1 items-center rounded-xl border border-white/60 bg-white/50 px-3 py-2.5 font-mono text-xs dark:border-white/10 dark:bg-slate-800/50">
        <span>{shown}</span>
        {typing && <span className="dv-caret" aria-hidden="true" />}
      </div>
    </div>
  );
}

function useTypewriter(text: string, speedMs = 90) {
  const reducedMotion = useReducedMotion();
  const [count, setCount] = useState(0);

  useEffect(() => {
    setCount(0);
    if (reducedMotion || text.length === 0) {
      setCount(text.length);
      return;
    }
    const id = window.setInterval(() => {
      setCount((v) => {
        if (v >= text.length) {
          window.clearInterval(id);
          return v;
        }
        return v + 1;
      });
    }, speedMs);
    return () => window.clearInterval(id);
  }, [text, speedMs, reducedMotion]);

  return { shown: text.slice(0, count), typing: count > 0 && count < text.length };
}
