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

function AppContent() {
  const { t } = useTranslation();
  const { handleError } = useErrorHandler();
  const [settingsOpen, setSettingsOpen] = useState(false);
  /** §1："添加设备"浮层（已连接时再配对一台）。无 latch——主视图纯派生。 */
  const [addDeviceOpen, setAddDeviceOpen] = useState(false);
  const startAttemptedRef = useRef(false);

  const serverState = useServerState();
  const settings = useAppSettings();
  const startServer = useStartServer();
  const { resolvedTheme } = useTheme();
  useAutoUpdate();

  // 应用启动时自动启动连接服务（仅一次）。
  //
  // start_server 在 Rust 侧一并拉起常驻子系统：heartbeat（注册/心跳/token
  // 分发）、信令监督（SSE 订阅 + WebRTC 应答面）、注入队列——webview 只是
  // 视图，不参与信令链路（托盘隐藏/休眠唤醒不再影响连接恢复）。
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
