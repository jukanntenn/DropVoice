import { useEffect, useMemo, useState } from 'react';
import { useReducedMotion } from 'framer-motion';
import { Monitor, Send, Smartphone } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Card, ConnectionStatus, cn } from '@dropvoice/ui';

import { phrasesFor } from './phrases';
import { demoFrame, demoStillFrame, type DemoFrame } from './script';

const TICK_MS = 80;

/**
 * Hero 活体演示（设计定稿：双屏实况）。
 * 手机玻璃卡逐字浮现台词 → 发送钮按下、粒子沿连接线流向电脑卡 →
 * 电脑卡光标处逐字上屏 → 静止 → 换下一句循环。
 * 时间线是纯函数（demo/script.ts），这里只做采样渲染；
 * prefers-reduced-motion 时定格终态。
 */
export function DemoStage() {
  const { t, i18n } = useTranslation();
  const phrases = useMemo(() => phrasesFor(i18n.language), [i18n.language]);
  const reducedMotion = useReducedMotion();
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (reducedMotion) {
      return;
    }
    const start = performance.now();
    const id = window.setInterval(() => {
      if (!document.hidden) {
        setElapsed(performance.now() - start);
      }
    }, TICK_MS);
    return () => window.clearInterval(id);
  }, [reducedMotion]);

  const frame = reducedMotion ? demoStillFrame(phrases) : demoFrame(elapsed, phrases);

  return (
    <div
      className="animate-slide-up mx-auto flex w-full max-w-sm flex-col items-center"
      aria-label={t('landing:hero.sub1')}
    >
      <PhoneCard frame={frame} label={t('landing:demo.phone')} />
      <FlowConnector sending={frame.phase === 'sending'} />
      <PcCard frame={frame} label={t('landing:demo.pc')} hint={t('landing:demo.injectHint')} />
    </div>
  );
}

function PhoneCard({ frame, label }: { frame: DemoFrame; label: string }) {
  const listening = frame.phase === 'listening';
  const sending = frame.phase === 'sending';

  return (
    <Card className="w-64 self-end p-4 md:mr-6">
      <div className="flex items-center justify-between gap-2">
        <span className="flex items-center gap-1.5 text-xs font-medium text-neutral-500 dark:text-neutral-400">
          <Smartphone className="h-3.5 w-3.5" aria-hidden="true" />
          {label}
        </span>
        <ConnectionStatus status="connected" latency={12} />
      </div>

      <div className="mt-3 flex min-h-11 items-center rounded-xl border border-white/60 bg-white/50 px-3 py-2 text-sm dark:border-white/10 dark:bg-slate-800/50">
        <span>{frame.phrase.slice(0, frame.phoneChars)}</span>
        {listening && <span className="dv-caret" aria-hidden="true" />}
      </div>

      <div
        className={cn('mt-3 flex items-end justify-between gap-3', listening && 'dv-wave-running')}
      >
        <div
          className="from-primary/60 text-primary dark:text-primary-300 flex h-10 items-center gap-1"
          aria-hidden="true"
        >
          {WAVE_HEIGHTS.map((h, i) => (
            <span
              key={i}
              className="dv-wave-bar w-1 rounded-full bg-current"
              style={{ height: h, animationDelay: `${i * 0.12}s` }}
            />
          ))}
        </div>
        <span
          className={cn(
            'from-primary shadow-primary/30 flex h-12 w-12 items-center justify-center rounded-full bg-gradient-to-br to-teal-500 text-white shadow-lg transition-transform duration-200 active:scale-95',
            sending && 'scale-90'
          )}
          aria-hidden="true"
        >
          <Send className="h-5 w-5" />
        </span>
      </div>
    </Card>
  );
}

/** 手机 → 电脑 的连接线；sending 阶段渲染 3 颗流动粒子。 */
function FlowConnector({ sending }: { sending: boolean }) {
  return (
    <div className="relative h-14 w-px" aria-hidden="true">
      <div className="border-primary/40 absolute inset-0 border-l border-dashed" />
      {sending && (
        <div className="absolute left-1/2 top-1 flex -translate-x-1/2 flex-col items-center gap-1.5">
          <span className="dv-particle bg-primary-500 h-1.5 w-1.5 rounded-full" />
          <span
            className="dv-particle bg-primary-400 h-1.5 w-1.5 rounded-full"
            style={{ animationDelay: '0.15s' }}
          />
          <span
            className="dv-particle h-1.5 w-1.5 rounded-full bg-cyan-400"
            style={{ animationDelay: '0.3s' }}
          />
        </div>
      )}
    </div>
  );
}

function PcCard({ frame, label, hint }: { frame: DemoFrame; label: string; hint: string }) {
  const showCaret = frame.phase === 'typing' || frame.phase === 'resting';

  return (
    <Card className="w-72 self-start p-4 md:ml-6">
      <div className="flex items-center justify-between gap-2">
        <span className="flex items-center gap-1.5 text-xs font-medium text-neutral-500 dark:text-neutral-400">
          <Monitor className="h-3.5 w-3.5" aria-hidden="true" />
          {label}
        </span>
      </div>

      <div className="mt-3 flex min-h-11 items-center rounded-xl border border-white/60 bg-white/50 px-3 py-2 text-sm dark:border-white/10 dark:bg-slate-800/50">
        {frame.pcChars === 0 ? (
          <span className="select-none text-neutral-400 dark:text-neutral-500">▏</span>
        ) : (
          <>
            <span>{frame.phrase.slice(0, frame.pcChars)}</span>
            {showCaret && <span className="dv-caret" aria-hidden="true" />}
          </>
        )}
      </div>

      <p className="mt-2 text-xs text-neutral-400 dark:text-neutral-500">{hint}</p>
    </Card>
  );
}

const WAVE_HEIGHTS = ['10px', '18px', '28px', '16px', '24px', '12px'];
