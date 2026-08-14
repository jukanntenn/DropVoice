import { z } from 'zod';

import { ERROR_I18N_MAP, isErrorCode, SUGGESTION_I18N_MAP, type ErrorCode } from './codes';

/**
 * Wire format produced by the Rust `AppError` Serialize impl.
 *
 * See `apps/desktop/src-tauri/src/error.rs` for the canonical definition.
 */
const ErrorSchema = z.object({
  code: z.string(),
  message: z.string(),
  context: z.record(z.string(), z.unknown()).optional().default({}),
});

export type ParsedError = z.infer<typeof ErrorSchema>;

export interface FormattedError {
  title: string;
  description: string;
  suggestion?: string;
  code: ErrorCode | string;
}

/**
 * Parse an unknown error payload (typically from a Tauri invoke rejection)
 * into the structured {@link ParsedError} shape. Non-conforming payloads are
 * wrapped as `INTERNAL_ERROR`.
 */
export function parseError(error: unknown): ParsedError {
  if (typeof error === 'string') {
    try {
      const parsed = JSON.parse(error) as unknown;
      const result = ErrorSchema.safeParse(parsed);
      if (result.success) {
        return result.data;
      }
    } catch {
      return {
        code: 'INTERNAL_ERROR',
        message: error,
        context: {},
      };
    }
  }

  if (error && typeof error === 'object') {
    const result = ErrorSchema.safeParse(error);
    if (result.success) {
      return result.data;
    }
  }

  return {
    code: 'INTERNAL_ERROR',
    message: error instanceof Error ? error.message : String(error ?? 'Unknown error'),
    context: {},
  };
}

type TranslationFn = (key: string, params?: Record<string, unknown>) => string;

/**
 * Format a parsed error for display, resolving i18n keys via the provided
 * translation function. Always returns a non-empty title and description.
 */
export function formatError(
  error: unknown,
  t: TranslationFn,
  fallbackMessage?: string
): FormattedError {
  const parsed = parseError(error);
  const code = parsed.code;
  const i18nKey = isErrorCode(code) ? ERROR_I18N_MAP[code] : 'errors:internalError';
  const context = parsed.context ?? {};

  const title = t(i18nKey, context) || parsed.message || fallbackMessage || 'Error';
  const description = parsed.message || t(i18nKey, context) || fallbackMessage || '';

  let suggestion: string | undefined;
  // Suggestions are heuristic-based on the error code.
  const suggestionKey = resolveSuggestionKey(code);
  if (suggestionKey) {
    const suggestionI18nKey = SUGGESTION_I18N_MAP[suggestionKey];
    if (suggestionI18nKey) {
      suggestion = t(suggestionI18nKey);
    }
  }

  return {
    title,
    description,
    suggestion,
    code,
  };
}

function resolveSuggestionKey(code: string): string | undefined {
  switch (code) {
    case 'NETWORK_UNREACHABLE':
      return 'check_network';
    case 'CONNECTION_REFUSED':
      return 'check_server_running';
    case 'CONNECTION_TIMEOUT':
      return 'retry_later';
    case 'PORT_IN_USE':
      return 'change_port';
    case 'MAX_DEVICES_REACHED':
      return 'remove_device';
    default:
      return undefined;
  }
}
