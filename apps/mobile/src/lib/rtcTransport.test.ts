import { describe, it, expect } from 'vitest';

import { offerStatusToErrorCode, parseQrPayload } from './rtcTransport';

describe('parseQrPayload', () => {
  it('parses a valid dropvoice:// payload', () => {
    const payload =
      'dropvoice://pair?code=123456&device=550e8400-e29b-41d4-a716-446655440000&name=My%20PC';
    const result = parseQrPayload(payload);
    expect(result).toEqual({
      code: '123456',
      device: '550e8400-e29b-41d4-a716-446655440000',
      name: 'My PC',
    });
  });

  it('parses payload without name', () => {
    const payload = 'dropvoice://pair?code=654321&device=abc-123';
    const result = parseQrPayload(payload);
    expect(result).toEqual({
      code: '654321',
      device: 'abc-123',
      name: undefined,
    });
  });

  it('returns null for non-dropvoice scheme', () => {
    expect(parseQrPayload('http://example.com/pair?code=123')).toBeNull();
    expect(parseQrPayload('ws://localhost:38425')).toBeNull();
  });

  it('returns null when code is missing', () => {
    expect(parseQrPayload('dropvoice://pair?device=abc')).toBeNull();
  });

  it('returns null when device is missing', () => {
    expect(parseQrPayload('dropvoice://pair?code=123456')).toBeNull();
  });

  it('returns null for malformed payload', () => {
    expect(parseQrPayload('not a url')).toBeNull();
    expect(parseQrPayload('')).toBeNull();
  });

  it('decodes URL-encoded name', () => {
    const payload = 'dropvoice://pair?code=111111&device=dev-1&name=Offi%20PC';
    const result = parseQrPayload(payload);
    expect(result?.name).toBe('Offi PC');
  });
});

describe('offerStatusToErrorCode (§7 含 503 fast-fail)', () => {
  it('503 → DESKTOP_OFFLINE（§7 桌面离线 fast-fail）', () => {
    expect(offerStatusToErrorCode(503)).toBe('DESKTOP_OFFLINE');
  });

  it('404 → DEVICE_NOT_FOUND', () => {
    expect(offerStatusToErrorCode(404)).toBe('DEVICE_NOT_FOUND');
  });

  it('429 → RATE_LIMITED', () => {
    expect(offerStatusToErrorCode(429)).toBe('RATE_LIMITED');
  });

  it('2xx / 其它状态 → null（成功或泛化错误，不映射具名 code）', () => {
    expect(offerStatusToErrorCode(200)).toBeNull();
    expect(offerStatusToErrorCode(500)).toBeNull();
    expect(offerStatusToErrorCode(400)).toBeNull();
  });
});
