import { useEffect, useState } from 'react';
import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import { changeLanguage, LANGUAGE_OPTIONS, type SupportedLanguage } from '@dropvoice/i18n';
import { themeAtom, type Theme } from '@dropvoice/core';
import { Button, Dialog, Input, Select, useErrorHandler } from '@dropvoice/ui';

import { useSetInputDelay, useSetLanguage, useSetTheme } from '../hooks/useAppSettings';
import type { Settings } from '../lib/invoke';

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
  settings: Settings | undefined;
}

const THEME_OPTIONS = [
  { value: 'light', label: 'settings:themeLight' },
  { value: 'dark', label: 'settings:themeDark' },
  { value: 'system', label: 'settings:themeSystem' },
] as const;

export function SettingsDialog({ open, onClose, settings }: SettingsDialogProps) {
  const { t } = useTranslation();
  const { handleError } = useErrorHandler();
  const [theme, setThemeAtom] = useAtom(themeAtom);
  const setLanguageMutation = useSetLanguage();
  const setThemeMutation = useSetTheme();
  const setInputDelayMutation = useSetInputDelay();
  const [delayValue, setDelayValue] = useState('');

  // 设置加载后同步本地延迟输入框。
  useEffect(() => {
    if (settings?.delay_ms !== undefined) {
      setDelayValue(String(settings.delay_ms));
    }
  }, [settings?.delay_ms]);

  const handleLanguageChange = async (lang: string) => {
    try {
      await changeLanguage(lang as SupportedLanguage);
    } catch (error) {
      handleError(error);
      return;
    }
    try {
      await setLanguageMutation.mutateAsync(lang);
    } catch (error) {
      handleError(error);
    }
  };

  const handleThemeChange = (value: string) => {
    setThemeAtom(value as Theme);
    setThemeMutation.mutate(value, { onError: (error) => handleError(error) });
  };

  const handleDelayBlur = () => {
    const num = Number(delayValue);
    if (Number.isNaN(num) || num < 0 || num > 5000) return;
    setInputDelayMutation.mutate(num, { onError: (error) => handleError(error) });
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
            value={settings?.language ?? 'en'}
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
            min={0}
            max={5000}
            value={delayValue}
            onChange={(e) => setDelayValue(e.target.value)}
            onBlur={handleDelayBlur}
          />
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
