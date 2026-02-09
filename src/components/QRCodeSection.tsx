import { QRCodeSVG } from "qrcode.react";
import { useTranslation } from "react-i18next";

export interface QRCodeSectionProps {
  url: string;
}

export default function QRCodeSection({ url }: QRCodeSectionProps) {
  const { t } = useTranslation();

  return (
    <div className="flex w-full flex-col items-center gap-4 pt-2">
      <p className="text-sm font-medium text-muted-foreground dark:text-slate-300">
        {t("connection.scanQR")}
      </p>
      <div className="relative">
        {/* Glow effect behind QR code */}
        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-primary/20 to-cyan-400/20 blur-xl" />

        {/* QR code container */}
        <div className="relative flex aspect-square w-48 items-center justify-center rounded-2xl border border-slate-200 bg-white shadow-lg dark:border-slate-600">
          <QRCodeSVG
            value={url}
            size={155}
            bgColor="transparent"
            fgColor="#0f172a"
            level="M"
          />
        </div>
      </div>
    </div>
  );
}
