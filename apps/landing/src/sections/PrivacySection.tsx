import { ArrowRight, Cloud, Monitor, Smartphone } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Card } from '@dropvoice/ui';

import { SectionKicker } from '../components/SectionKicker';

/**
 * 深潜 A · 隐私路径图（设计定稿）：云输入法两跳 vs DropVoice P2P 直连，
 * 加一条诚实的握手说明。层级用线型表达：dashed=经过服务器，实线 teal=直连。
 */
export function PrivacySection() {
  const { t } = useTranslation();

  return (
    <section id="privacy">
      <div className="mx-auto grid max-w-6xl items-center gap-12 px-4 py-20 md:py-28 lg:grid-cols-2">
        <div>
          <SectionKicker>{t('landing:privacy.kicker')}</SectionKicker>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight text-neutral-900 md:text-3xl dark:text-white">
            {t('landing:privacy.title')}
          </h2>
          <p className="mt-5 leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:privacy.p1')}
          </p>
          <p className="mt-3 leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:privacy.p2')}
          </p>
          <p className="bg-warning-500/10 mt-5 rounded-lg px-3 py-2 text-xs leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:privacy.note')}
          </p>
        </div>

        <Card className="flex flex-col gap-8 p-6 md:p-8">
          <div>
            <p className="mb-3 font-mono text-xs text-neutral-400 dark:text-neutral-500">
              {t('landing:privacy.cloudLabel')}
            </p>
            <div className="flex items-center gap-2">
              <Node icon={<Smartphone className="h-4 w-4" />} label={t('landing:privacy.phone')} />
              <DashedLink />
              <Node
                icon={<Cloud className="h-4 w-4" />}
                label={t('landing:privacy.cloud')}
                tone="danger"
              />
              <DashedLink />
              <Node icon={<Monitor className="h-4 w-4" />} label={t('landing:privacy.pc')} />
            </div>
            <p className="mt-3 text-xs text-neutral-500 dark:text-neutral-400">
              {t('landing:privacy.cloudCaption')}
            </p>
          </div>

          <div>
            <p className="text-primary-700 dark:text-primary-300 mb-3 font-mono text-xs font-medium">
              {t('landing:privacy.p2pLabel')}
            </p>
            <div className="flex items-center gap-2">
              <Node icon={<Smartphone className="h-4 w-4" />} label={t('landing:privacy.phone')} />
              <SolidLink />
              <Node icon={<Monitor className="h-4 w-4" />} label={t('landing:privacy.pc')} />
            </div>
            <p className="mt-3 text-xs font-medium text-neutral-600 dark:text-neutral-300">
              {t('landing:privacy.p2pCaption')}
            </p>
            <p className="mt-1 text-xs text-neutral-400 dark:text-neutral-500">
              {t('landing:privacy.handshakeCaption')}
            </p>
          </div>
        </Card>
      </div>
    </section>
  );
}

function Node({ icon, label, tone }: { icon: React.ReactNode; label: string; tone?: 'danger' }) {
  return (
    <div className="flex w-16 flex-col items-center gap-1.5">
      <span
        className={
          tone === 'danger'
            ? 'border-danger-500/30 bg-danger-500/10 text-danger-600 dark:text-danger-500 flex h-10 w-10 items-center justify-center rounded-full border'
            : 'flex h-10 w-10 items-center justify-center rounded-full border border-white/60 bg-white/70 text-neutral-600 dark:border-white/10 dark:bg-slate-800/70 dark:text-neutral-300'
        }
      >
        {icon}
      </span>
      <span className="text-[10px] text-neutral-500 dark:text-neutral-400">{label}</span>
    </div>
  );
}

function DashedLink() {
  return (
    <div className="flex flex-1 items-center" aria-hidden="true">
      <div className="flex-1 border-t border-dashed border-neutral-300 dark:border-neutral-600" />
      <ArrowRight className="h-4 w-4 text-neutral-400 dark:text-neutral-500" />
    </div>
  );
}

function SolidLink() {
  return (
    <div className="flex flex-1 items-center" aria-hidden="true">
      <div className="from-primary h-0.5 flex-1 rounded-full bg-gradient-to-r to-cyan-400" />
      <ArrowRight className="text-primary-600 dark:text-primary-400 h-4 w-4" />
    </div>
  );
}
