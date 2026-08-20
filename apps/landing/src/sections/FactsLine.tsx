import { useTranslation } from 'react-i18next';

import { SITE } from '../lib/site';

/**
 * 事实行——hero 与原理区之间的一行 mono 小字（设计定稿）。
 * 只放可验证的真值，不做 "Trusted by"。star 数刻意不展示：
 * 真实数字尚小，等它配得上页面时再加。
 */
export function FactsLine() {
  const { t } = useTranslation();

  return (
    <div className="mx-auto max-w-6xl px-4 py-8">
      <p className="text-center font-mono text-xs tracking-wider text-neutral-500 dark:text-neutral-400">
        {t('landing:facts.oss')} · v{SITE.version} · {t('landing:facts.stack')}
      </p>
    </div>
  );
}
