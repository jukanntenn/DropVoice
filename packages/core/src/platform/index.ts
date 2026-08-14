/**
 * Browser environment detection per spec 08 section 2.7 and spec 11 section 2.6.
 *
 * Used by the mobile PWA to decide whether Service Worker / camera / install
 * prompts are available.
 */

export interface EnvironmentInfo {
  protocol: 'http' | 'https';
  hostname: string;
  isSecureContext: boolean;
  isLocalhost: boolean;
  isLAN: boolean;
  serviceWorkerSupported: boolean;
  cameraSupported: boolean;
}

export type PWAInstallStrategy = 'pwa' | 'bookmark' | 'none';

export type Platform = 'ios' | 'android' | 'desktop' | 'unknown';

function detectPlatformFromUA(ua: string): Platform {
  if (/iPad|iPhone|iPod/.test(ua)) {
    return 'ios';
  }
  if (/Android/.test(ua)) {
    return 'android';
  }
  if (/Windows|Macintosh|Linux/.test(ua)) {
    return 'desktop';
  }
  return 'unknown';
}

export function detectPlatform(): Platform {
  if (typeof navigator === 'undefined') {
    return 'unknown';
  }
  return detectPlatformFromUA(navigator.userAgent);
}

export function isSecureContext(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  return Boolean(window.isSecureContext);
}

export function isLocalhost(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  const { hostname } = window.location;
  return (
    hostname === 'localhost' ||
    hostname === '127.0.0.1' ||
    hostname === '::1' ||
    hostname === '0.0.0.0'
  );
}

function isLANAddress(hostname: string): boolean {
  // IPv4 private ranges.
  if (hostname.startsWith('10.')) return true;
  if (hostname.startsWith('192.168.')) return true;
  // 172.16.0.0 – 172.31.255.255
  const match = hostname.match(/^172\.(\d+)\./);
  if (match) {
    const second = Number(match[1]);
    if (second >= 16 && second <= 31) return true;
  }
  // Link-local.
  if (hostname.startsWith('169.254.')) return true;
  // IPv6 unique local (fc00::/7).
  if (/^f[cd]/i.test(hostname)) return true;
  return false;
}

export function isCameraSupported(): boolean {
  if (typeof navigator === 'undefined') {
    return false;
  }
  return (
    Boolean(navigator.mediaDevices && typeof navigator.mediaDevices.getUserMedia === 'function') &&
    isSecureContext()
  );
}

export function isServiceWorkerSupported(): boolean {
  if (typeof navigator === 'undefined') {
    return false;
  }
  return 'serviceWorker' in navigator && isSecureContext();
}

export function detectEnvironment(): EnvironmentInfo {
  if (typeof window === 'undefined') {
    return {
      protocol: 'http',
      hostname: '',
      isSecureContext: false,
      isLocalhost: false,
      isLAN: false,
      serviceWorkerSupported: false,
      cameraSupported: false,
    };
  }

  const { protocol, hostname } = window.location;
  return {
    protocol: protocol === 'https:' ? 'https' : 'http',
    hostname,
    isSecureContext: isSecureContext(),
    isLocalhost: isLocalhost(),
    isLAN: isLANAddress(hostname),
    serviceWorkerSupported: isServiceWorkerSupported(),
    cameraSupported: isCameraSupported(),
  };
}

export function getPWAInstallStrategy(): PWAInstallStrategy {
  const env = detectEnvironment();
  if (env.isSecureContext || env.isLocalhost) {
    if (env.serviceWorkerSupported) {
      return 'pwa';
    }
    return 'bookmark';
  }
  return 'none';
}
