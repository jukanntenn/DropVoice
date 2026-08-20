import { useTranslation } from 'react-i18next';

import { changeLanguage, isSupportedLanguage, LANGUAGE_OPTIONS } from '@dropvoice/i18n';
import { Select } from '@dropvoice/ui';

import logo from '../assets/logo.png';
import { SITE } from '../lib/site';

/** 页脚——单行极简（设计定稿）：品牌 + 链接 + 语言切换 + 版权。 */
export function Footer() {
  const { t, i18n } = useTranslation();

  return (
    <footer className="border-t border-white/40 dark:border-white/10">
      <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-4 py-8 md:flex-row">
        <a href="#top" className="flex items-center gap-2" aria-label="DropVoice">
          <span className="from-primary flex h-7 w-7 items-center justify-center rounded-lg bg-gradient-to-br to-teal-400">
            <img src={logo} alt="" className="h-4.5 w-4.5 rounded" />
          </span>
          <span className="text-sm font-semibold tracking-tight text-neutral-900 dark:text-white">
            DropVoice
          </span>
        </a>

        <nav
          className="flex flex-wrap items-center justify-center gap-4 text-sm text-neutral-500 dark:text-neutral-400"
          aria-label="Footer"
        >
          <a
            href={SITE.repo}
            target="_blank"
            rel="noreferrer"
            className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
          >
            GitHub
          </a>
          <a
            href={`${SITE.repo}/blob/main/LICENSE`}
            target="_blank"
            rel="noreferrer"
            className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
          >
            {t('landing:footer.license')}
          </a>
          <a
            href="#self-host"
            className="hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
          >
            {t('landing:footer.selfHost')}
          </a>
        </nav>

        <div className="flex items-center gap-4">
          <Select
            aria-label="Language"
            value={i18n.language}
            onValueChange={(v) => {
              if (isSupportedLanguage(v)) {
                void changeLanguage(v);
              }
            }}
            options={LANGUAGE_OPTIONS}
            className="h-9 w-auto text-xs"
          />
          <span className="text-xs text-neutral-400 dark:text-neutral-500">
            {t('landing:footer.copyright')}
          </span>
        </div>
      </div>
    </footer>
  );
}
