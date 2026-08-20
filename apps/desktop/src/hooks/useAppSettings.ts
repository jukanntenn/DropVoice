import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import { useAtom } from 'jotai';

import { QUERY_KEYS, inputDelayAtom, languageAtom, themeAtom, type Theme } from '@dropvoice/core';
import type { SupportedLanguage } from '@dropvoice/i18n';

import { tauriInvoke } from '../lib/invoke';

/**
 * 读取后端持久化设置，并在数据到达时同步 jotai 客户端状态原子
 * （themeAtom / languageAtom / inputDelayAtom）。
 */
export function useAppSettings() {
  const query = useQuery({
    queryKey: QUERY_KEYS.settings,
    queryFn: tauriInvoke.getSettings,
  });

  const [, setThemeAtom] = useAtom(themeAtom);
  const [, setLanguageAtom] = useAtom(languageAtom);
  const [, setInputDelayAtom] = useAtom(inputDelayAtom);

  // 后端设置加载后同步到 jotai 原子，供 useTheme 和 UI 组件使用。
  useEffect(() => {
    if (!query.data) return;
    setThemeAtom(query.data.theme as Theme);
    setLanguageAtom(query.data.language as SupportedLanguage);
    setInputDelayAtom(query.data.delay_ms);
  }, [query.data, setThemeAtom, setLanguageAtom, setInputDelayAtom]);

  return query;
}

/**
 * 修改语言。成功后更新 jotai 原子并刷新设置缓存。
 */
export function useSetLanguage() {
  const queryClient = useQueryClient();
  const [, setLanguageAtom] = useAtom(languageAtom);
  return useMutation({
    mutationFn: (language: string) => tauriInvoke.setLanguage(language),
    onSuccess: (_data, language) => {
      setLanguageAtom(language as SupportedLanguage);
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.settings });
    },
  });
}

/**
 * 修改主题。成功后更新 jotai 原子并刷新设置缓存。
 */
export function useSetTheme() {
  const queryClient = useQueryClient();
  const [, setThemeAtom] = useAtom(themeAtom);
  return useMutation({
    mutationFn: (theme: string) => tauriInvoke.setTheme(theme),
    onSuccess: (_data, theme) => {
      setThemeAtom(theme as Theme);
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.settings });
    },
  });
}

/**
 * 修改输入注入延迟。成功后更新 jotai 原子并刷新设置缓存。
 */
export function useSetInputDelay() {
  const queryClient = useQueryClient();
  const [, setInputDelayAtom] = useAtom(inputDelayAtom);
  return useMutation({
    mutationFn: (delay_ms: number) => tauriInvoke.setInputDelay(delay_ms),
    onSuccess: (_data, delay_ms) => {
      setInputDelayAtom(delay_ms);
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.settings });
    },
  });
}

/**
 * 修改开机自启动。成功后刷新设置缓存（autostart 由后端实时持有）。
 */
export function useSetAutostart() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabled: boolean) => tauriInvoke.setAutostart(enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.settings });
    },
  });
}
