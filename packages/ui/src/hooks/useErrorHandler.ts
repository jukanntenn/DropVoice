/**
 * useErrorHandler — error → toast bridge (spec 04 section 4.4).
 *
 * Parses unknown errors via `@dropvoice/core/errors`, resolves an i18n title
 * and description, surfaces them as a sonner error toast, and logs the parsed
 * payload in development.
 */
import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

import { formatError, parseError } from '@dropvoice/core/errors';

export interface UseErrorHandlerReturn {
  handleError: (error: unknown, fallbackMessage?: string) => void;
}

export function useErrorHandler(): UseErrorHandlerReturn {
  const { t } = useTranslation();

  const handleError = useCallback(
    (error: unknown, fallbackMessage?: string) => {
      const { title, description, suggestion } = formatError(error, t);
      toast.error(title, {
        description: suggestion || description || fallbackMessage,
      });
      if (import.meta.env.DEV) {
        // eslint-disable-next-line no-console -- diagnostic only in dev
        console.error('[Error]', parseError(error));
      }
    },
    [t]
  );

  return { handleError };
}
