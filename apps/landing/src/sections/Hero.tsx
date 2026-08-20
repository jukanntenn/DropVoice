import { Download } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { buttonVariants } from '@dropvoice/ui';

import { DemoStage } from '../demo/DemoStage';
import { QrPopover } from '../components/QrPopover';
import { SITE } from '../lib/site';

/** Hero——双屏实况（设计定稿 A）：左文案 + 双 CTA，右活体演示。 */
export function Hero() {
  const { t } = useTranslation();

  return (
    <section className="mx-auto max-w-6xl px-4 pb-14 pt-14 md:pb-20 md:pt-24">
      <div className="grid items-center gap-12 lg:grid-cols-[1.1fr_1fr]">
        <div className="animate-slide-up">
          <h1 className="text-4xl font-semibold leading-[1.15] tracking-tight text-neutral-900 md:text-5xl dark:text-white">
            {t('landing:hero.title1')}
            <br />
            {t('landing:hero.title2')}
          </h1>

          <p className="mt-5 max-w-md text-lg leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:hero.sub1')}
            <br />
            {t('landing:hero.sub2')}
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-3">
            <a href={SITE.releases} className={buttonVariants({ size: 'lg' })}>
              <Download className="h-4 w-4" aria-hidden="true" />
              {t('landing:hero.downloadCta')}
            </a>
            <QrPopover />
          </div>

          <p className="mt-4 text-sm text-neutral-500 dark:text-neutral-400">
            {t('landing:hero.platformLine')}
          </p>
        </div>

        <DemoStage />
      </div>
    </section>
  );
}
