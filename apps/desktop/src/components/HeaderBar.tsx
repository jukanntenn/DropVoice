import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import { changeLanguage, LANGUAGE_OPTIONS, type SupportedLanguage } from '@dropvoice/i18n';
import { themeAtom } from '@dropvoice/core';
import { Header, useErrorHandler, type ThemeMode } from '@dropvoice/ui';

import { useSetLanguage, useSetTheme } from '../hooks/useAppSettings';

interface HeaderBarProps {
  onOpenSettings: () => void;
}

export function HeaderBar({ onOpenSettings }: HeaderBarProps) {
  const { t, i18n } = useTranslation();
  const [theme, setThemeAtom] = useAtom(themeAtom);
  const { handleError } = useErrorHandler();
  const setLanguageMutation = useSetLanguage();
  const setThemeMutation = useSetTheme();

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
      // 后端持久化失败不阻塞 i18n 切换，仅提示用户。
      handleError(error);
    }
  };

  const handleThemeChange = (next: ThemeMode) => {
    setThemeAtom(next);
    setThemeMutation.mutate(next, {
      onError: (error) => handleError(error),
    });
  };

  return (
    <Header
      title={t('common:app.name')}
      version="0.2.0"
      languages={LANGUAGE_OPTIONS}
      language={i18n.language}
      onLanguageChange={(lang) => void handleLanguageChange(lang)}
      theme={theme}
      onThemeChange={handleThemeChange}
      onOpenSettings={onOpenSettings}
    />
  );
}
