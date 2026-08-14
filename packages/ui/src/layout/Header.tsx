/**
 * Header — liquid-glass app header with brand tile, version pill, and
 * language/theme/settings controls (spec 07 section 2.4).
 *
 * Pure presentational component: the app owns language/theme state and passes
 * the current value plus a change handler. Language options are supplied by the
 * app (e.g. derived from `@dropvoice/i18n`) so this package stays decoupled
 * from the i18n package's runtime. The header is transparent and sticky,
 * floating over the liquid gradient; controls use inverted-color tooltips.
 */
import { Monitor, Moon, Settings, Sun } from 'lucide-react';
import type { ReactNode } from 'react';

import logoSrc from '../assets/logo.png';
import { cn } from '../lib/cn';
import { Badge } from '../primitives/Badge';
import { Button } from '../primitives/Button';
import { Select } from '../primitives/Select';
import { Tooltip } from '../primitives/Tooltip';

export type ThemeMode = 'light' | 'dark' | 'system';

export interface HeaderLanguageOption {
  value: string;
  label: string;
}

export interface HeaderProps {
  title: string;
  version?: string;
  /** Language options rendered in the selector. */
  languages?: HeaderLanguageOption[];
  language?: string;
  onLanguageChange?: (language: string) => void;
  theme?: ThemeMode;
  onThemeChange?: (theme: ThemeMode) => void;
  onOpenSettings?: () => void;
  className?: string;
  /** Optional trailing actions rendered after the settings button. */
  children?: ReactNode;
}

const THEME_ORDER: ThemeMode[] = ['light', 'dark', 'system'];

const ThemeIcon: Record<ThemeMode, typeof Sun> = {
  light: Sun,
  dark: Moon,
  system: Monitor,
};

export function Header({
  title,
  version,
  languages,
  language,
  onLanguageChange,
  theme,
  onThemeChange,
  onOpenSettings,
  className,
  children,
}: HeaderProps) {
  const currentTheme: ThemeMode = theme ?? 'system';
  const ThemeIconComponent = ThemeIcon[currentTheme];

  const cycleTheme = () => {
    const index = THEME_ORDER.indexOf(currentTheme);
    const next = THEME_ORDER[(index + 1) % THEME_ORDER.length];
    onThemeChange?.(next);
  };

  return (
    <header
      className={cn(
        'sticky top-0 z-20 flex h-14 items-center justify-between border-b border-transparent px-4 backdrop-blur-none',
        className
      )}
    >
      <div className="flex items-center gap-2">
        <span className="from-primary shadow-primary/25 flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br to-teal-400 text-white shadow-lg">
          <img src={logoSrc} alt="DropVoice" className="h-6 w-6 rounded-lg" />
        </span>
        <h1 className="text-lg font-semibold tracking-tight text-neutral-900 dark:text-white">
          {title}
        </h1>
        {version && (
          <Badge
            variant="outline"
            className="bg-primary/15 text-primary dark:bg-primary/20 dark:text-primary-300 ml-0.5 rounded-md font-mono text-[10px] font-medium"
          >
            v{version}
          </Badge>
        )}
      </div>

      <div className="flex items-center gap-1">
        {onLanguageChange && languages && languages.length > 0 && (
          <Select
            aria-label="Select language"
            value={language}
            onValueChange={(v) => v && onLanguageChange(v)}
            className="h-9 w-auto gap-2 rounded-xl border-transparent bg-transparent px-3 shadow-none hover:bg-white/60 dark:hover:bg-slate-800/60"
            options={languages}
          />
        )}
        {onThemeChange && (
          <Tooltip content={`Theme: ${currentTheme}`}>
            <Button
              variant="ghost"
              size="icon"
              onClick={cycleTheme}
              aria-label={`Theme: ${currentTheme}`}
              className="h-9 w-9 rounded-xl"
            >
              <ThemeIconComponent className="h-4.5 w-4.5" />
            </Button>
          </Tooltip>
        )}
        {onOpenSettings && (
          <Tooltip content="Settings">
            <Button
              variant="ghost"
              size="icon"
              onClick={onOpenSettings}
              aria-label="Settings"
              className="h-9 w-9 rounded-xl"
            >
              <Settings className="h-4.5 w-4.5" />
            </Button>
          </Tooltip>
        )}
        {children}
      </div>
    </header>
  );
}
