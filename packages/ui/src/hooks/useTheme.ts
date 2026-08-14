/**
 * useTheme — applies the jotai `themeAtom` to `<html>` and resolves the
 * effective colour scheme. Shared by the desktop and mobile apps so they
 * don't each carry a private copy (spec 02 §4.3).
 *
 * - When theme is 'system', subscribes to the `prefers-color-scheme: dark`
 *   media query and re-resolves when it changes.
 * - Returns `{ theme, resolvedTheme }` so consumers (e.g. Toaster) can pick
 *   a variant without re-reading the atom.
 */
import { useEffect, useState } from 'react';
import { useAtomValue } from 'jotai';

import { themeAtom, type Theme } from '@dropvoice/core';

export function useTheme(): { theme: Theme; resolvedTheme: 'light' | 'dark' } {
  const theme = useAtomValue(themeAtom);
  const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>(() => {
    if (typeof window === 'undefined' || !window.matchMedia) return 'light';
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });

  // Compute resolvedTheme and, under 'system', react to OS theme changes.
  useEffect(() => {
    const resolve = () => {
      if (theme === 'system') {
        setResolvedTheme(
          window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
        );
      } else {
        setResolvedTheme(theme);
      }
    };

    resolve();

    if (theme === 'system') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      mq.addEventListener('change', resolve);
      return () => mq.removeEventListener('change', resolve);
    }
    return undefined;
  }, [theme]);

  // Apply resolvedTheme to <html>'s class list.
  useEffect(() => {
    const root = document.documentElement;
    if (resolvedTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [resolvedTheme]);

  return { theme, resolvedTheme };
}
