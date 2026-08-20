import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { Card, ConnectionStatus, QRCode } from '@dropvoice/ui';

import { ScenarioTabs } from '../components/ScenarioTabs';
import { SectionKicker } from '../components/SectionKicker';
import { SITE } from '../lib/site';

/**
 * 三步原理——异构三卡（设计定稿）：控制台微缩 / 扫码动作 / 三场景接收端。
 * 三张卡视觉形态各异，不做同构 icon 卡。
 */
export function HowItWorks() {
  const { t } = useTranslation();

  return (
    <section id="how-it-works">
      <div className="mx-auto max-w-6xl px-4 py-20 md:py-28">
        <SectionKicker>{t('landing:how.kicker')}</SectionKicker>
        <h2 className="mt-3 text-2xl font-semibold tracking-tight text-neutral-900 md:text-3xl dark:text-white">
          {t('landing:how.title')}
        </h2>

        <div className="mt-10 grid gap-6 md:grid-cols-3">
          <StepCard step="01" title={t('landing:how.s1t')} desc={t('landing:how.s1d')}>
            <div className="flex min-h-36 flex-col items-center justify-center gap-3 rounded-xl border border-white/60 bg-white/60 p-3 dark:border-white/10 dark:bg-slate-800/60">
              <QRCode value={SITE.pwaUrl} size={56} glow={false} />
              <ConnectionStatus status="connected" />
            </div>
          </StepCard>

          <StepCard step="02" title={t('landing:how.s2t')} desc={t('landing:how.s2d')}>
            <div className="relative flex min-h-36 items-center justify-center overflow-hidden rounded-xl border border-white/60 bg-white/60 p-3 dark:border-white/10 dark:bg-slate-800/60">
              <ScanFrame>
                <QRCode value={SITE.pwaUrl} size={56} glow={false} />
              </ScanFrame>
            </div>
          </StepCard>

          <StepCard step="03" title={t('landing:how.s3t')} desc={t('landing:how.s3d')}>
            <ScenarioTabs
              scenarios={[
                { id: 'chat', label: t('landing:how.tabChat'), text: t('landing:how.chatText') },
                { id: 'doc', label: t('landing:how.tabDoc'), text: t('landing:how.docText') },
                { id: 'term', label: t('landing:how.tabTerm'), text: t('landing:how.termText') },
              ]}
            />
          </StepCard>
        </div>
      </div>
    </section>
  );
}

function StepCard({
  step,
  title,
  desc,
  children,
}: {
  step: string;
  title: string;
  desc: string;
  children: ReactNode;
}) {
  return (
    <Card className="flex flex-col p-5">
      <div className="flex items-baseline gap-2">
        <span className="text-primary-600 dark:text-primary-400 font-mono text-xs">{step}</span>
        <h3 className="font-semibold text-neutral-900 dark:text-white">{title}</h3>
      </div>
      <p className="mt-1 text-sm text-neutral-500 dark:text-neutral-400">{desc}</p>
      <div className="mt-4 flex-1">{children}</div>
    </Card>
  );
}

/** 扫码动作微缩：四角取景框 + 扫掠线。 */
function ScanFrame({ children }: { children: ReactNode }) {
  const corner = 'border-primary-600/70 dark:border-primary-400/70 absolute h-3.5 w-3.5';
  return (
    <div className="relative rounded-lg p-3">
      <span className={`${corner} left-0 top-0 border-l-2 border-t-2`} aria-hidden="true" />
      <span className={`${corner} right-0 top-0 border-r-2 border-t-2`} aria-hidden="true" />
      <span className={`${corner} bottom-0 left-0 border-b-2 border-l-2`} aria-hidden="true" />
      <span className={`${corner} bottom-0 right-0 border-b-2 border-r-2`} aria-hidden="true" />
      <div
        className="from-primary/60 to-primary dark:to-primary-400 dv-scanline absolute left-3 right-3 top-2 h-0.5 rounded-full bg-gradient-to-r"
        aria-hidden="true"
      />
      <div className="text-primary-600 dark:text-primary-400">{children}</div>
    </div>
  );
}
