import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { isCameraSupported, MAX_DEVICES, type ScannedDevice } from '@dropvoice/core';
import { Button, Dialog, Input, useErrorHandler } from '@dropvoice/ui';

import { parseQrPayload } from '../lib/rtcTransport';

interface AddDeviceModalProps {
  open: boolean;
  onClose: () => void;
  onAddDevice: (scanned: ScannedDevice) => 'added' | 'switched' | false;
  /** 当前设备数量，用于判断是否达到上限。 */
  deviceCount: number;
}

/**
 * 添加设备对话框：扫码（HTTPS / localhost 可用）+ 手动粘贴 QR 载荷。
 *
 * 扫码使用 `html5-qrcode`，解析 `dropvoice://pair?code=&device=&name=` 载荷（§6.5）。
 * 提取 code + device_id + name 后调用 onAddDevice。
 */
export function AddDeviceModal({ open, onClose, onAddDevice, deviceCount }: AddDeviceModalProps) {
  const { t } = useTranslation();
  const { handleError } = useErrorHandler();
  const [manualPayload, setManualPayload] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const scannerRef = useRef<import('html5-qrcode').Html5Qrcode | null>(null);
  const scannerContainerId = 'qr-scan-container';

  const cameraAvailable = isCameraSupported();

  // 关闭对话框时停止扫码并清理状态。
  useEffect(() => {
    if (!open) {
      void stopScanning();
      setManualPayload('');
      setError(null);
    }
    return () => {
      void stopScanning();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  async function stopScanning() {
    if (scannerRef.current) {
      try {
        await scannerRef.current.stop();
        scannerRef.current.clear();
      } catch {
        // 忽略：摄像头可能已停止。
      }
      scannerRef.current = null;
    }
    setScanning(false);
  }

  async function startScanning() {
    if (!cameraAvailable) return;
    setError(null);
    // 先渲染容器：Html5Qrcode 构造函数要求元素已存在于 DOM
    // （此前先构造后 setScanning，容器未渲染 → 构造抛错 → 按钮点击毫无反应）。
    setScanning(true);
    try {
      const { Html5Qrcode } = await import('html5-qrcode');
      const scanner = new Html5Qrcode(scannerContainerId);
      scannerRef.current = scanner;
      await scanner.start(
        { facingMode: 'environment' },
        { fps: 10, qrbox: 200 },
        (decodedText: string) => {
          void handleScannedPayload(decodedText);
        },
        () => {
          // 每帧扫描失败时静默。
        }
      );
    } catch (err) {
      // 失败时清理扫描器 + 恢复 UI。
      if (scannerRef.current) {
        await scannerRef.current.stop().catch(() => undefined);
        scannerRef.current.clear();
        scannerRef.current = null;
      }
      setScanning(false);
      handleError(err);
      setError(t('devices:cameraNotAvailable'));
    }
  }

  /** 处理扫码或手动输入的 QR 载荷，解析后调用 onAddDevice。 */
  function handleScannedPayload(payload: string) {
    void stopScanning();
    const parsed = parseQrPayload(payload);
    if (!parsed) {
      setError(t('devices:invalidUrl'));
      return;
    }
    const result = onAddDevice(parsed);
    if (result === 'added' || result === 'switched') {
      onClose();
    } else if (result === false) {
      if (deviceCount >= MAX_DEVICES) {
        setError(t('devices:maxLimit'));
      } else {
        setError(t('devices:invalidUrl'));
      }
    }
  }

  function handleSubmitPayload(event: React.FormEvent) {
    event.preventDefault();
    setError(null);
    const trimmed = manualPayload.trim();
    if (!trimmed) {
      setError(t('devices:invalidUrl'));
      return;
    }
    handleScannedPayload(trimmed);
  }

  return (
    <Dialog open={open} onClose={onClose} title={t('devices:addDevice')}>
      <div className="flex flex-col gap-4">
        {!cameraAvailable && (
          <p className="text-sm text-neutral-500 dark:text-neutral-400">
            {t('devices:cameraRequiresHTTPS')}
          </p>
        )}

        {cameraAvailable && (
          <div className="flex flex-col gap-2">
            {!scanning ? (
              <Button variant="outline" onClick={() => void startScanning()}>
                {t('devices:scanToConnect')}
              </Button>
            ) : (
              <div className="flex flex-col gap-2">
                <div
                  id={scannerContainerId}
                  className="h-64 w-full overflow-hidden rounded-md bg-neutral-100 dark:bg-neutral-800"
                />
                <Button variant="ghost" onClick={() => void stopScanning()}>
                  {t('common:actions.cancel')}
                </Button>
              </div>
            )}
          </div>
        )}

        <form onSubmit={handleSubmitPayload} className="flex flex-col gap-2">
          <label htmlFor="add-device-payload" className="text-sm font-medium">
            {t('devices:manualInput')}
          </label>
          <Input
            id="add-device-payload"
            value={manualPayload}
            placeholder="dropvoice://pair?code=..."
            onChange={(e) => setManualPayload(e.target.value)}
            autoComplete="off"
            autoCapitalize="off"
            spellCheck={false}
          />
          {error && (
            <p className="text-danger-600 dark:text-danger-400 text-sm" role="alert">
              {error}
            </p>
          )}
          <div className="mt-2 flex justify-end gap-2">
            <Button variant="outline" type="button" onClick={onClose}>
              {t('common:actions.cancel')}
            </Button>
            <Button variant="primary" type="submit">
              {t('common:actions.add')}
            </Button>
          </div>
        </form>
      </div>
    </Dialog>
  );
}
