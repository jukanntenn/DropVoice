import { useState } from 'react';
import { Smartphone } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button, QRCode } from '@dropvoice/ui';

import { SITE } from '../lib/site';

/**
 * Hero 次级 CTA——手机扫码弹泡（设计定稿）。
 * 复用产品的 QRCode 组件（白卡 + teal 光晕），hover 展开、点击切换（触屏），
 * 扫码直达手机 PWA。
 */
export function QrPopover() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  return (
    <div
      className="relative inline-block"
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
    >
      <Button variant="outline" size="lg" aria-expanded={open} onClick={() => setOpen((v) => !v)}>
        <Smartphone className="h-4 w-4" aria-hidden="true" />
        {t('landing:hero.mobileCta')}
      </Button>

      {open && (
        <div className="animate-scale-in shadow-glass dark:shadow-glass-dark absolute right-0 top-full z-40 mt-3 rounded-2xl border border-white/60 bg-white/85 p-4 backdrop-blur-md dark:border-white/10 dark:bg-slate-900/85">
          <QRCode value={SITE.pwaUrl} size={112} />
          <p className="mt-3 max-w-40 text-center text-xs text-neutral-600 dark:text-neutral-300">
            {t('landing:hero.qrCaption')}
          </p>
        </div>
      )}
    </div>
  );
}
