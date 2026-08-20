import { useEffect, useRef, useState } from 'react';

import { listen } from '@tauri-apps/api/event';
import { Provider as JotaiProvider } from 'jotai';
import { QueryClientProvider } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';

import { parseError } from '@dropvoice/core/errors';
import { QUERY_KEYS, queryClient } from '@dropvoice/core';
import type { ConnectionInfo } from '@dropvoice/core';
import { Dialog, LiquidBackground, Toaster, useErrorHandler, useTheme } from '@dropvoice/ui';

import { ConnectedStatusCard } from './components/ConnectedStatusCard';
import { HeaderBar } from './components/HeaderBar';
import { LanWarningBanner } from './components/LanWarningBanner';
import { PairingContent } from './components/PairingContent';
import { PairingView } from './components/PairingView';
import { SettingsDialog } from './components/SettingsDialog';
import { useAppSettings } from './hooks/useAppSettings';
import { useAutoUpdate } from './hooks/useAutoUpdate';
import { useServerState, useStartServer } from './hooks/useServerState';
import { useWebRTC } from './hooks/useWebRTC';
import { tauriInvoke } from './lib/invoke';

/** 从 QR 载荷提取 device_id（dropvoice://pair?device=<id>&...）。 */
function parseDeviceIdFromQr(qrPayload: string): string | null {
  try {
    const url = new URL(qrPayload);
    return url.searchParams.get('device');
  } catch {
    return null;
  }
}

function AppContent() {
  const { t } = useTranslation();
  const { handleError } = useErrorHandler();
  const [settingsOpen, setSettingsOpen] = useState(false);
  /** §1："添加设备"浮层（已连接时再配对一台）。无 latch——主视图纯派生。 */
  const [addDeviceOpen, setAddDeviceOpen] = useState(false);
  const startAttemptedRef = useRef(false);
  /** 已启动信令客户端的 device_id（幂等：设备不变不重启）。 */
  const startedDeviceIdRef = useRef<string | null>(null);

  const serverState = useServerState();
  const settings = useAppSettings();
  const startServer = useStartServer();
  const { resolvedTheme } = useTheme();
  useAutoUpdate();
  const webrtc = useWebRTC();

  // 应用启动时自动启动连接服务（仅一次）。
  useEffect(() => {
    if (startAttemptedRef.current) return;
    startAttemptedRef.current = true;
    startServer.mutateAsync().catch((error) => {
      // 服务可能已在运行，这不是真正的错误，静默处理。
      const parsed = parseError(error);
      if (parsed.code !== 'SERVER_ALREADY_RUNNING') {
        handleError(error);
      }
    });
  }, [startServer, handleError]);

  // 服务启动后：解析 device_id，监听首个 pairing_token，启动 WebRTC 信令客户端。
  // SignalingClient.start 内部已监听后续 pairing_token 刷新（重建 SSE）。
  //
  // 幂等 + 自愈设计：
  // - 设备不变时不重启客户端；设备变化（或数据瞬时丢失后恢复）时自动重建。
  // - 不设一次性 ref 守卫——旧版"只启动一次 + cleanup 调 webrtc.stop()"在
  //   get_connection_info 瞬时失败（running 变 undefined）时会把客户端永久停掉。
  useEffect(() => {
    const qrPayload = serverState.data?.qr_payload;
    if (!qrPayload || !serverState.data?.running) return;
    const deviceId = parseDeviceIdFromQr(qrPayload);
    if (!deviceId) return;

    // 已为同一设备启动过 → 幂等返回。
    if (startedDeviceIdRef.current === deviceId) return;
    startedDeviceIdRef.current = deviceId;

    let unlistenFn: (() => void) | null = null;
    let started = false;
    let pollTimer: number | undefined;
    let pollTimeout: number | undefined;

    const startOnce = (token: string) => {
      if (started) return;
      started = true;
      webrtc.start(deviceId, token);
      if (pollTimer !== undefined) window.clearInterval(pollTimer);
      if (pollTimeout !== undefined) window.clearTimeout(pollTimeout);
      if (unlistenFn) {
        unlistenFn();
        unlistenFn = null;
      }
    };

    // 注册 listener：首个 pairing_token 到达后启动 SSE 客户端，并取消自身。
    listen<string>('pairing_token', (event) => {
      if (event.payload) startOnce(event.payload);
    })
      .then((fn) => {
        unlistenFn = fn;
      })
      .catch((err) => console.error('[app] listen pairing_token failed', err));

    // 兜底：heartbeat 注册成功即 emit，可能早于本 listener 注册（Tauri 事件
    // 不排队、不重发，事件会丢失）。300ms 轮询 Rust 已持久化的 token，最迟 8s。
    pollTimer = window.setInterval(async () => {
      if (started) {
        window.clearInterval(pollTimer);
        return;
      }
      try {
        const token = await tauriInvoke.getPairingToken();
        if (token) startOnce(token);
      } catch {
        // server 未就绪，继续轮询。
      }
    }, 300);
    pollTimeout = window.setTimeout(() => {
      window.clearInterval(pollTimer);
    }, 8000);

    return () => {
      window.clearInterval(pollTimer);
      window.clearTimeout(pollTimeout);
      if (unlistenFn) unlistenFn();
      // 设备变化（或重新挂载）时由下一次 effect 运行重建客户端，此处不永久停。
    };
  }, [serverState.data?.qr_payload, serverState.data?.running, webrtc]);

  // §6 配对码轮换事件总线：后端定时器到点轮换后 push `pairing_code_rotated`，
  // 前端即时更新连接信息缓存（QR/链接/重连码）。1s 轮询仍兜底 active_connections
  // /clients/queue_depth（无专用事件）。
  useEffect(() => {
    let un: (() => void) | null = null;
    listen<{ code: string; qr_payload: string }>('pairing_code_rotated', (event) => {
      queryClient.setQueryData<ConnectionInfo>(QUERY_KEYS.connectionInfo, (old) =>
        old
          ? { ...old, pairing_code: event.payload.code, qr_payload: event.payload.qr_payload }
          : old
      );
    })
      .then((fn) => {
        un = fn;
      })
      .catch((err) => console.error('[app] listen pairing_code_rotated failed', err));
    return () => {
      un?.();
    };
  }, []);

  const connectionInfo = serverState.data;
  const activeConnections = connectionInfo?.active_connections ?? 0;
  // §1：主视图纯派生——0 连接显示首配，>0 显示状态卡。无 latch。
  const hasConnections = connectionInfo?.running === true && activeConnections > 0;

  // §1/§0.3：订阅 client_registered 事件——任意新连接（新配对或既有重连）即关闭
  // "添加设备"浮层。client_id 不可预知，故不区分；用户重新点"添加设备"即可。
  useEffect(() => {
    let un: (() => void) | null = null;
    listen<{ client_id: string }>('client_registered', () => setAddDeviceOpen(false))
      .then((fn) => {
        un = fn;
      })
      .catch((err) => console.error('[app] listen client_registered failed', err));
    return () => {
      un?.();
    };
  }, []);

  // 注入结果事件总线：后台队列异步注入完成后 push，前端 console 可观测
  // （成功/失败 + 字符数 + 耗时；失败时配合 Rust 侧 warn! 定位根因）。
  useEffect(() => {
    let unCompleted: (() => void) | null = null;
    let unFailed: (() => void) | null = null;
    listen<{ client_id: string; success: boolean; chars: number; elapsed_ms: number }>(
      'injection_completed',
      (event) => {
        console.debug('[injection] completed', event.payload);
      }
    )
      .then((fn) => {
        unCompleted = fn;
      })
      .catch((err) => console.error('[app] listen injection_completed failed', err));
    listen<{ client_id: string; success: boolean; chars: number; elapsed_ms: number }>(
      'injection_failed',
      (event) => {
        console.warn('[injection] failed', event.payload);
      }
    )
      .then((fn) => {
        unFailed = fn;
      })
      .catch((err) => console.error('[app] listen injection_failed failed', err));
    return () => {
      unCompleted?.();
      unFailed?.();
    };
  }, []);

  return (
    <div className="relative flex min-h-screen flex-col">
      <LiquidBackground />
      <HeaderBar onOpenSettings={() => setSettingsOpen(true)} />
      <main className="relative flex flex-1 flex-col items-center justify-start gap-8 px-4 py-8 md:px-12">
        <LanWarningBanner />
        {hasConnections ? (
          <ConnectedStatusCard
            activeConnections={activeConnections}
            clients={connectionInfo?.clients}
            queueDepth={connectionInfo?.queue_depth}
            onAddDevice={() => setAddDeviceOpen(true)}
          />
        ) : (
          <PairingView
            connectionInfo={connectionInfo}
            onStart={() => startServer.mutateAsync().catch(handleError)}
            isStarting={startServer.isPending}
          />
        )}
      </main>
      <SettingsDialog
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        settings={settings.data}
      />
      {/* §1："添加设备"浮层（已连接时再配对一台）。复用 PairingContent；
          client_registered 事件自动关闭（见上方 listener）。 */}
      <Dialog
        open={addDeviceOpen}
        onClose={() => setAddDeviceOpen(false)}
        title={t('common:connection.addDevice')}
      >
        <PairingContent connectionInfo={connectionInfo} />
      </Dialog>
      <Toaster theme={resolvedTheme} />
    </div>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <JotaiProvider>
        <AppContent />
      </JotaiProvider>
    </QueryClientProvider>
  );
}
