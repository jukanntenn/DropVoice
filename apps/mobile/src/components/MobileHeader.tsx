import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import { changeLanguage, LANGUAGE_OPTIONS, type SupportedLanguage } from '@dropvoice/i18n';
import { themeAtom } from '@dropvoice/core';
import { Header, useErrorHandler, type ThemeMode } from '@dropvoice/ui';

interface MobileHeaderProps {
  onOpenSettings: () => void;
}

export function MobileHeader({ onOpenSettings }: MobileHeaderProps) {
  const { t, i18n } = useTranslation();
  const [theme, setThemeAtom] = useAtom(themeAtom);
  const { handleError } = useErrorHandler();

  const handleLanguageChange = async (lang: string) => {
    try {
      await changeLanguage(lang as SupportedLanguage);
    } catch (error) {
      handleError(error);
    }
  };

  const handleThemeChange = (next: ThemeMode) => {
    setThemeAtom(next);
  };

  return (
    <Header
      title={t('common:app.name')}
      languages={LANGUAGE_OPTIONS}
      language={i18n.language}
      onLanguageChange={(lang) => void handleLanguageChange(lang)}
      theme={theme}
      onThemeChange={handleThemeChange}
      onOpenSettings={onOpenSettings}
    />
  );
}
