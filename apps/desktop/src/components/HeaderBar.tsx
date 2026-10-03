import { useEffect, useState } from 'react';

import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import { getVersion } from '@tauri-apps/api/app';

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
  // 版本来自 tauri.conf.json（pnpm version:sync 的单一真源）；仅在非 Tauri
  // 环境（如浏览器 dev）取不到时显示 dev。
  const [version, setVersion] = useState('dev');
  useEffect(() => {
    let cancelled = false;
    getVersion()
      .then((v) => {
        if (!cancelled) setVersion(v);
      })
      .catch(() => {
        if (!cancelled) setVersion('dev');
      });
    return () => {
      cancelled = true;
    };
  }, []);

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
      version={version}
      languages={LANGUAGE_OPTIONS}
      language={i18n.language}
      onLanguageChange={(lang) => void handleLanguageChange(lang)}
      theme={theme}
      onThemeChange={handleThemeChange}
      onOpenSettings={onOpenSettings}
    />
  );
}
