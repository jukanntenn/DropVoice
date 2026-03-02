import { useEffect, useRef } from "react";
import { Html5Qrcode } from "html5-qrcode";

interface QRScannerProps {
  onScan: (text: string) => void;
  onError?: (error: string) => void;
}

export function QRScanner({ onScan, onError }: QRScannerProps) {
  const isStoppedRef = useRef(false);

  useEffect(() => {
    isStoppedRef.current = false;
    const scanner = new Html5Qrcode("qr-scanner");

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
          onError?.(String(errorMessage));
        },
      )
      .catch((err) => {
        onError?.(String(err));
      });

    return () => {
      isStoppedRef.current = true;
      scanner.stop().catch(() => {});
    };
  }, [onError, onScan]);

  return <div id="qr-scanner" className="w-full overflow-hidden rounded-2xl bg-black" />;
}
