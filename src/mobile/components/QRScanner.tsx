import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Html5Qrcode } from "html5-qrcode";

interface QRScannerProps {
  onScan: (text: string) => void;
  onError?: (error: string) => void;
  continuous?: boolean;
}

export function QRScanner({ onScan, onError, continuous }: QRScannerProps) {
  const { t } = useTranslation();
  const scannerRef = useRef<Html5Qrcode | null>(null);
  const isStoppedRef = useRef(false);
  const initTimerRef = useRef<number | null>(null);
  const cooldownRef = useRef(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    isStoppedRef.current = false;
    setIsLoading(true);
    const initWithRetry = (attempt = 0) => {
      if (isStoppedRef.current) return;

      const element = document.getElementById("qr-scanner");
      if (!element) {
        if (attempt < 20) {
          initTimerRef.current = window.setTimeout(() => initWithRetry(attempt + 1), 50);
        } else {
          setIsLoading(false);
          onError?.(t("devices.cameraInitFailed", { reason: "Scanner element not found" }));
        }
        return;
      }

      try {
        const scanner = new Html5Qrcode("qr-scanner");
        scannerRef.current = scanner;

        scanner
          .start(
            { facingMode: "environment" },
            { fps: 10, qrbox: { width: 250, height: 250 } },
            (decodedText) => {
              if (isStoppedRef.current) return;
              if (cooldownRef.current) return;

              onScan(decodedText);

              if (continuous) {
                cooldownRef.current = true;
                setTimeout(() => {
                  cooldownRef.current = false;
                }, 1000);
              } else {
                isStoppedRef.current = true;
                setIsLoading(false);
                scanner.stop().catch(() => {});
              }
            },
            (errorMessage) => {
              const msg = String(errorMessage);
              if (msg.includes("No barcode") || msg.includes("NotFoundException")) return;
              if (msg.includes("NotAllowedError") || msg.includes("Permission denied")) {
                onError?.(t("devices.cameraPermissionDenied"));
                return;
              }
              if (msg.includes("NotFoundError")) {
                onError?.(t("devices.cameraNotFound"));
                return;
              }
              if (msg.includes("NotReadableError")) {
                onError?.(t("devices.cameraInUse"));
                return;
              }
              onError?.(msg);
            },
          )
          .then(() => {
            setIsLoading(false);
          })
          .catch((err) => {
            setIsLoading(false);
            const msg =
              err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
            if (msg.includes("NotAllowedError") || msg.includes("Permission denied")) {
              onError?.(t("devices.cameraPermissionDenied"));
              return;
            }
            if (msg.includes("NotFoundError")) {
              onError?.(t("devices.cameraNotFound"));
              return;
            }
            if (msg.includes("NotReadableError")) {
              onError?.(t("devices.cameraInUse"));
              return;
            }
            onError?.(t("devices.cameraInitFailed", { reason: msg }));
          });
      } catch (err) {
        setIsLoading(false);
        onError?.(
          t("devices.cameraInitFailed", {
            reason: err instanceof Error ? err.message : String(err),
          }),
        );
      }
    };

    initWithRetry();

    return () => {
      isStoppedRef.current = true;
      if (initTimerRef.current !== null) {
        window.clearTimeout(initTimerRef.current);
        initTimerRef.current = null;
      }
      const scanner = scannerRef.current;
      scannerRef.current = null;
      scanner?.stop().catch(() => {});
    };
  }, [onError, onScan, t, continuous]);

  return (
    <div className="relative w-full aspect-square overflow-hidden rounded-2xl bg-black">
      {isLoading && (
        <div className="absolute inset-0 z-10 flex items-center justify-center bg-slate-900">
          <div className="text-center">
            <div className="mx-auto h-8 w-8 animate-spin rounded-full border-2 border-white border-t-transparent" />
            <p className="mt-3 text-sm text-white">{t("devices.startingCamera")}</p>
          </div>
        </div>
      )}
      <div id="qr-scanner" className="h-full w-full" />
    </div>
  );
}
