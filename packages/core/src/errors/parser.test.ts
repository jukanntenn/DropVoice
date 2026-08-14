import { describe, it, expect } from 'vitest';

import { ERROR_CODES, ERROR_I18N_MAP, isErrorCode } from './codes';
import { formatError, parseError } from './parser';

describe('error codes', () => {
  it('includes all expected codes', () => {
    expect(ERROR_CODES).toContain('PORT_IN_USE');
    expect(ERROR_CODES).toContain('MAX_DEVICES_REACHED');
    expect(ERROR_CODES).toContain('INTERNAL_ERROR');
  });

  it('maps every code to an i18n key', () => {
    for (const code of ERROR_CODES) {
      expect(ERROR_I18N_MAP[code]).toMatch(/^errors:/);
    }
  });

  it('isErrorCode narrows type', () => {
    expect(isErrorCode('PORT_IN_USE')).toBe(true);
    expect(isErrorCode('UNKNOWN')).toBe(false);
  });
});

describe('parseError', () => {
  it('parses a structured error object', () => {
    const parsed = parseError({
      code: 'PORT_IN_USE',
      message: 'Port in use',
      context: { port: 80 },
    });
    expect(parsed.code).toBe('PORT_IN_USE');
    expect(parsed.context).toEqual({ port: 80 });
  });

  it('parses a JSON string', () => {
    const parsed = parseError(JSON.stringify({ code: 'TEXT_EMPTY', message: 'empty' }));
    expect(parsed.code).toBe('TEXT_EMPTY');
  });

  it('wraps arbitrary strings as INTERNAL_ERROR', () => {
    const parsed = parseError('boom');
    expect(parsed.code).toBe('INTERNAL_ERROR');
    expect(parsed.message).toBe('boom');
  });

  it('wraps unknown shapes as INTERNAL_ERROR', () => {
    const parsed = parseError({ foo: 'bar' });
    expect(parsed.code).toBe('INTERNAL_ERROR');
  });
});

describe('formatError', () => {
  const t = (key: string, params?: Record<string, unknown>) => {
    if (params && 'port' in params) return `${key}:${String(params.port)}`;
    return key;
  };

  it('returns title and description using i18n', () => {
    const result = formatError(
      { code: 'PORT_IN_USE', message: 'Port in use', context: { port: 80 } },
      t
    );
    expect(result.title).toContain('errors:portInUse');
    expect(result.code).toBe('PORT_IN_USE');
  });

  it('includes a suggestion for known codes', () => {
    const result = formatError({ code: 'PORT_IN_USE', message: 'x', context: {} }, t);
    expect(result.suggestion).toBe('errors:suggestion.changePort');
  });

  it('falls back to internal error for unknown codes', () => {
    const result = formatError({ code: 'WHATEVER', message: 'x', context: {} }, t);
    expect(result.code).toBe('WHATEVER');
    expect(result.title).toBe('errors:internalError');
  });
});
