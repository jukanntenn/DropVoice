import { useEffect, useRef } from "react";
import { Html5Qrcode } from "html5-qrcode";

interface QRScannerProps {
  onScan: (text: string) => void;
  onError?: (error: string) => void;
}

export function QRScanner({ onScan, onError }: QRScannerProps) {
  const scannerRef = useRef<Html5Qrcode | null>(null);
  const isStoppedRef = useRef(false);
  const initTimerRef = useRef<number | null>(null);

  useEffect(() => {
    isStoppedRef.current = false;
    const initWithRetry = (attempt = 0) => {
      if (isStoppedRef.current) return;

      const element = document.getElementById("qr-scanner");
      if (!element) {
        if (attempt < 20) {
          initTimerRef.current = window.setTimeout(() => initWithRetry(attempt + 1), 50);
        } else {
          onError?.("Scanner element not found after retries");
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
              isStoppedRef.current = true;
              onScan(decodedText);
              scanner.stop().catch(() => {});
            },
            (errorMessage) => {
              const msg = String(errorMessage);
              if (msg.includes("No barcode") || msg.includes("NotFoundException")) return;
              onError?.(msg);
            },
          )
          .catch((err) => {
            const msg =
              err && typeof err === "object" && "message" in err ? String(err.message) : String(err);
            onError?.(`Camera init failed: ${msg}`);
          });
      } catch (err) {
        onError?.(`Scanner init failed: ${err instanceof Error ? err.message : String(err)}`);
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
  }, [onError, onScan]);

  return (
    <div
      id="qr-scanner"
      className="w-full aspect-square bg-black rounded-2xl overflow-hidden"
    />
  );
}
