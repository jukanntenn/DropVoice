import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import { changeLanguage, LANGUAGE_OPTIONS, type SupportedLanguage } from '@dropvoice/i18n';
import { inputDelayAtom, themeAtom, type Theme } from '@dropvoice/core';
import { Button, Dialog, Input, Select, useErrorHandler } from '@dropvoice/ui';

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
}

const THEME_OPTIONS = [
  { value: 'light', label: 'settings:themeLight' },
  { value: 'dark', label: 'settings:themeDark' },
  { value: 'system', label: 'settings:themeSystem' },
] as const;

const APP_VERSION = '0.2.0';

/**
 * 设置对话框：语言 / 主题 / 输入延迟（只读，由桌面端控制） / About。
 */
export function SettingsDialog({ open, onClose }: SettingsDialogProps) {
  const { t, i18n } = useTranslation();
  const { handleError } = useErrorHandler();
  const [theme, setThemeAtom] = useAtom(themeAtom);
  const [inputDelay] = useAtom(inputDelayAtom);

  const handleLanguageChange = async (lang: string) => {
    try {
      await changeLanguage(lang as SupportedLanguage);
    } catch (error) {
      handleError(error);
    }
  };

  const handleThemeChange = (value: string) => {
    setThemeAtom(value as Theme);
  };

  return (
    <Dialog open={open} onClose={onClose} title={t('settings:title')}>
      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium" htmlFor="settings-language">
            {t('settings:language')}
          </label>
          <Select
            id="settings-language"
            value={i18n.language}
            options={LANGUAGE_OPTIONS}
            onValueChange={(v) => v && void handleLanguageChange(v)}
          />
        </div>
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium" htmlFor="settings-theme">
            {t('settings:theme')}
          </label>
          <Select
            id="settings-theme"
            value={theme}
            onValueChange={(v) => v && handleThemeChange(v)}
            options={THEME_OPTIONS.map((opt) => ({
              value: opt.value,
              label: t(opt.label),
            }))}
          />
        </div>
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium" htmlFor="settings-delay">
            {t('settings:inputDelay')}
          </label>
          <Input
            id="settings-delay"
            type="number"
            value={String(inputDelay)}
            readOnly
            aria-readonly
          />
          <p className="text-xs text-neutral-500 dark:text-neutral-400">
            {t('settings:inputDelay')} ({t('common:app.name')})
          </p>
        </div>
        <div className="border-t border-neutral-200 pt-3 dark:border-neutral-800">
          <p className="text-sm font-medium">{t('settings:about')}</p>
          <p className="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            {t('common:app.version', { version: APP_VERSION })}
          </p>
          <p className="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
            GitHub: https://github.com/dropvoice/dropvoice
          </p>
        </div>
      </div>
      <div className="mt-6 flex justify-end">
        <Button variant="outline" onClick={onClose}>
          {t('common:actions.close')}
        </Button>
      </div>
    </Dialog>
  );
}
