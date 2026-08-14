import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Provider as JotaiProvider } from 'jotai';
import { useAtom } from 'jotai';
import { useTranslation } from 'react-i18next';

import {
  type ScannedDevice,
  sendModeAtom,
  selectedDeviceIdsAtom,
  useDeviceManager,
} from '@dropvoice/core';
import { LiquidBackground, Toaster, toast, useTheme } from '@dropvoice/ui';

import { ActionButtons } from './components/ActionButtons';
import { AddDeviceModal } from './components/AddDeviceModal';
import { ConnectionStatusPanel } from './components/ConnectionStatusPanel';
import { DeviceSelectorPanel } from './components/DeviceSelectorPanel';
import { InstallPrompt } from './components/InstallPrompt';
import { MobileHeader } from './components/MobileHeader';
import { SettingsDialog } from './components/SettingsDialog';
import { SendModeSelector } from './components/SendModeSelector';
import { TextInputPanel } from './components/TextInputPanel';
import { useConnections } from './hooks/useConnections';
import { useDraft } from './hooks/useDraft';
import { getLastSent, setLastSent, setPairingCode } from './lib/storage';

const MAX_TEXT_LENGTH = 10000;

function AppContent() {
  const { t } = useTranslation();
  const { resolvedTheme } = useTheme();
  const { devices, activeDeviceId, setActiveDeviceId, addDevice, removeDevice } =
    useDeviceManager();

  const [sendMode, setSendMode] = useAtom(sendModeAtom);
  const [selectedDeviceIds, setSelectedDeviceIds] = useAtom(selectedDeviceIdsAtom);

  // §9：连接瞬态唯一真相源在此 hook（statuses）。Device 只含持久身份。
  const { statuses, connectDevice, disconnectDevice, retryDevice, sendToDevice, sendAll } =
    useConnections();

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [addDeviceOpen, setAddDeviceOpen] = useState(false);
  // 用户刚添加的设备 ID：等待 devices 列表更新后触发连接。
  const [pendingConnectDeviceId, setPendingConnectDeviceId] = useState<string | null>(null);

  const activeDevice = useMemo(
    () => devices.find((device) => device.id === activeDeviceId) ?? null,
    [devices, activeDeviceId]
  );

  // 草稿：跟随活跃设备切换。
  const [draft, setDraft] = useDraft(activeDeviceId);

  // 启动时自动连接 autoConnect=true 的设备（仅执行一次）。
  const autoConnectAttemptedRef = useRef(false);
  useEffect(() => {
    if (autoConnectAttemptedRef.current) return;
    if (devices.length === 0) return;
    autoConnectAttemptedRef.current = true;
    for (const device of devices) {
      if (device.autoConnect) {
        connectDevice(device);
      }
    }
  }, [devices, connectDevice]);

  // 当 devices 列表更新后，处理待连接的设备。
  useEffect(() => {
    if (!pendingConnectDeviceId) return;
    const target = devices.find((device) => device.id === pendingConnectDeviceId);
    if (target) {
      connectDevice(target);
      setPendingConnectDeviceId(null);
    }
  }, [pendingConnectDeviceId, devices, connectDevice]);

  // 活跃设备的完整连接状态（§9.6：面板收完整 ConnectionState）。
  const activeStatus = statuses[activeDeviceId ?? ''] ?? { status: 'idle' };

  const handleRetry = useCallback(() => {
    if (activeDeviceId) {
      retryDevice(activeDeviceId);
    }
  }, [activeDeviceId, retryDevice]);

  const handleAddDevice = useCallback(
    (scanned: ScannedDevice): 'added' | 'switched' | false => {
      const result = addDevice(scanned);
      if (result === 'added' || result === 'switched') {
        // 持久化扫码得到的配对码（§6.2），让 transport 自动用 code 连接。
        setPairingCode(scanned.device, scanned.code);
        // 触发 useEffect 在 devices 更新后连接。
        setPendingConnectDeviceId(scanned.device);
      }
      return result;
    },
    [addDevice]
  );

  const handleRemoveDevice = useCallback(
    (deviceId: string) => {
      disconnectDevice(deviceId);
      removeDevice(deviceId);
      if (activeDeviceId === deviceId) {
        const fallback = devices.find((device) => device.id !== deviceId) ?? null;
        setActiveDeviceId(fallback?.id ?? null);
      }
    },
    [activeDeviceId, devices, disconnectDevice, removeDevice, setActiveDeviceId]
  );

  const canSend = useMemo(() => {
    if (!draft.trim()) return false;
    if (devices.length === 0) return false;
    // 至少有一个设备已连接（认证状态由发送 API 二次校验）。
    return devices.some((device) => statuses[device.id]?.status === 'connected');
  }, [draft, devices, statuses]);

  const canRestore = useMemo(() => {
    if (!activeDeviceId) return false;
    return getLastSent(activeDeviceId).length > 0;
  }, [activeDeviceId]);

  const handleSend = useCallback(() => {
    const text = draft.trim();
    if (!text) return;

    let successCount = 0;
    if (sendMode === 'all') {
      successCount = sendAll(text);
    } else if (sendMode === 'selected') {
      for (const id of selectedDeviceIds) {
        if (sendToDevice(id, text)) successCount += 1;
      }
    } else if (activeDeviceId) {
      // active 模式：发送到活跃设备。
      if (sendToDevice(activeDeviceId, text)) {
        successCount = 1;
      }
    }

    if (successCount > 0) {
      if (activeDeviceId) {
        setLastSent(activeDeviceId, text);
      }
      // 发送成功后清空草稿。
      setDraft('');
      toast.success(t('common:actions.send'));
    } else {
      toast.error(t('errors:connectionLost'));
    }
  }, [activeDeviceId, draft, sendAll, sendToDevice, selectedDeviceIds, sendMode, setDraft, t]);

  const handleRestore = useCallback(() => {
    if (!activeDeviceId) return;
    const last = getLastSent(activeDeviceId);
    if (last) setDraft(last);
  }, [activeDeviceId, setDraft]);

  const handleClear = useCallback(() => {
    setDraft('');
  }, [setDraft]);

  const handleSelectDevice = useCallback(
    (deviceId: string) => {
      setActiveDeviceId(deviceId);
      // 切换活跃设备后，如未连接则尝试连接。
      const target = devices.find((device) => device.id === deviceId);
      if (target && statuses[deviceId]?.status !== 'connected') {
        connectDevice(target);
      }
    },
    [devices, statuses, connectDevice, setActiveDeviceId]
  );

  return (
    <div className="relative flex min-h-screen flex-col">
      <LiquidBackground />
      <MobileHeader onOpenSettings={() => setSettingsOpen(true)} />
      <main className="relative mx-auto flex w-full max-w-md flex-1 flex-col gap-3 px-5 pb-[calc(env(safe-area-inset-bottom)+1.75rem)] pt-3">
        <InstallPrompt />
        {devices.length === 0 ? (
          <div className="animate-slide-up shadow-glass dark:shadow-glass-dark mt-6 flex flex-col items-center gap-4 rounded-3xl border border-white/60 bg-white/70 p-8 text-center backdrop-blur-xl dark:border-white/10 dark:bg-slate-900/70">
            <p className="text-sm text-neutral-600 dark:text-slate-300">
              {t('devices:emptyState')}
            </p>
            <button
              type="button"
              onClick={() => setAddDeviceOpen(true)}
              className="from-primary shadow-primary/30 hover:shadow-primary/40 inline-flex items-center gap-2 rounded-full bg-gradient-to-br to-teal-500 px-5 py-2.5 text-sm font-medium text-white shadow-lg transition-all hover:shadow-xl active:scale-95"
            >
              {t('devices:addDevice')}
            </button>
          </div>
        ) : (
          <>
            <ConnectionStatusPanel
              activeDevice={activeDevice}
              status={activeStatus}
              onRetry={handleRetry}
            />
            <DeviceSelectorPanel
              devices={devices}
              activeDeviceId={activeDeviceId}
              onSelect={handleSelectDevice}
              onAddDevice={() => setAddDeviceOpen(true)}
              onRemoveDevice={handleRemoveDevice}
            />
            <SendModeSelector
              mode={sendMode}
              devices={devices}
              selectedDevices={selectedDeviceIds}
              onModeChange={setSendMode}
              onSelectionChange={setSelectedDeviceIds}
            />
            <TextInputPanel
              value={draft}
              onChange={setDraft}
              onSend={handleSend}
              maxLength={MAX_TEXT_LENGTH}
              canSend={canSend}
            />
            <ActionButtons
              onRestore={handleRestore}
              onSend={handleSend}
              onClear={handleClear}
              canSend={canSend}
              canRestore={canRestore}
            />
          </>
        )}
      </main>
      <SettingsDialog open={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <AddDeviceModal
        open={addDeviceOpen}
        onClose={() => setAddDeviceOpen(false)}
        onAddDevice={handleAddDevice}
        deviceCount={devices.length}
      />
      <Toaster theme={resolvedTheme} />
    </div>
  );
}

export default function App() {
  return (
    <JotaiProvider>
      <AppContent />
    </JotaiProvider>
  );
}
