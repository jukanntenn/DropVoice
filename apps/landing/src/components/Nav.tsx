import { Github, Moon, Sun } from 'lucide-react';
import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import { themeAtom } from '@dropvoice/core';
import { Button, buttonVariants } from '@dropvoice/ui';

import logo from '../assets/logo.png';
import { SITE } from '../lib/site';

/**
 * Landing 导航——镜像 app Header 的玻璃条语言（设计定稿：不做悬浮胶囊）。
 * 滚动时保持 sticky 玻璃；teal 按钮是全页常驻的唯一主动作（原则 4）。
 */
export function Nav() {
  const { t } = useTranslation();
  const [theme, setTheme] = useAtom(themeAtom);
  const isDark = theme === 'dark';

  return (
    <header className="sticky top-0 z-30 border-b border-white/40 bg-white/60 backdrop-blur-md dark:border-white/10 dark:bg-slate-950/60">
      <div className="mx-auto flex h-14 max-w-6xl items-center justify-between gap-3 px-4">
        <a href="#top" className="flex items-center gap-2" aria-label="DropVoice">
          <span className="from-primary shadow-primary/25 flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br to-teal-400 shadow-lg">
            <img src={logo} alt="" className="h-6 w-6 rounded-lg" />
          </span>
          <span className="text-lg font-semibold tracking-tight text-neutral-900 dark:text-white">
            DropVoice
          </span>
        </a>

        <nav className="hidden items-center gap-1 md:flex" aria-label="Sections">
          <a
            href="#how-it-works"
            className="rounded-xl px-3 py-2 text-sm text-neutral-600 transition-colors hover:bg-white/60 dark:text-neutral-300 dark:hover:bg-slate-800/60"
          >
            {t('landing:nav.how')}
          </a>
          <a
            href="#self-host"
            className="rounded-xl px-3 py-2 text-sm text-neutral-600 transition-colors hover:bg-white/60 dark:text-neutral-300 dark:hover:bg-slate-800/60"
          >
            {t('landing:nav.selfHost')}
          </a>
          <a
            href="#faq"
            className="rounded-xl px-3 py-2 text-sm text-neutral-600 transition-colors hover:bg-white/60 dark:text-neutral-300 dark:hover:bg-slate-800/60"
          >
            {t('landing:nav.faq')}
          </a>
        </nav>

        <div className="flex items-center gap-1">
          <a
            href={SITE.repo}
            target="_blank"
            rel="noreferrer"
            aria-label="GitHub"
            className={buttonVariants({ variant: 'ghost', size: 'icon' })}
          >
            <Github className="h-4.5 w-4.5" aria-hidden="true" />
          </a>
          <Button
            variant="ghost"
            size="icon"
            aria-label={isDark ? 'Light theme' : 'Dark theme'}
            onClick={() => setTheme(isDark ? 'light' : 'dark')}
            className="h-9 w-9 rounded-xl"
          >
            {isDark ? (
              <Sun className="h-4.5 w-4.5" aria-hidden="true" />
            ) : (
              <Moon className="h-4.5 w-4.5" aria-hidden="true" />
            )}
          </Button>
          <a href={SITE.releases} className={buttonVariants({ variant: 'primary', size: 'sm' })}>
            {t('landing:nav.download')}
          </a>
        </div>
      </div>
    </header>
  );
}
