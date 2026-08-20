import { Download } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { QRCode, buttonVariants } from '@dropvoice/ui';

import { PLATFORM_LINKS, SITE } from '../lib/site';

/**
 * 终章 CTA（设计定稿）：一句行动号召 + 平台按钮（ghost——teal 主按钮
 * 已在导航常驻，此处避免双主 CTA）+ 真 QR（产品 QRCode 组件，
 * 与 Hero 弹泡形成呼应）。
 */
export function FinalCta() {
  const { t } = useTranslation();

  return (
    <section className="mx-auto max-w-6xl px-4 py-20 md:py-28">
      <div className="flex flex-col items-center gap-8 text-center">
        <div>
          <h2 className="text-3xl font-semibold tracking-tight text-neutral-900 md:text-4xl dark:text-white">
            {t('landing:final.title')}
          </h2>
          <p className="mt-4 text-lg text-neutral-600 dark:text-neutral-300">
            {t('landing:final.sub')}
          </p>
        </div>

        <div className="flex flex-wrap items-center justify-center gap-3">
          {PLATFORM_LINKS.map((p) => (
            <a key={p.label} href={p.href} className={buttonVariants({ variant: 'outline' })}>
              <Download className="h-4 w-4" aria-hidden="true" />
              {p.label}
            </a>
          ))}
        </div>

        <div className="flex flex-col items-center">
          <QRCode value={SITE.pwaUrl} size={96} />
          <p className="mt-3 text-xs text-neutral-500 dark:text-neutral-400">
            {t('landing:final.scanCaption')}
          </p>
        </div>
      </div>
    </section>
  );
}
