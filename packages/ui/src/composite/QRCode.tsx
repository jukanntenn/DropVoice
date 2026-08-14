/**
 * QR code composite built on `qrcode.react`. Renders the code on a pure-white
 * rounded card so it scans on any background, with a soft teal→cyan glow
 * behind it (the liquid-glass signature).
 */
import { QRCodeSVG } from 'qrcode.react';

import { cn } from '../lib/cn';

export interface QRCodeProps {
  value: string;
  size?: number;
  level?: 'L' | 'M' | 'Q' | 'H';
  bgColor?: string;
  fgColor?: string;
  includeMargin?: boolean;
  /** Show the ambient teal→cyan glow behind the card. Defaults to true. */
  glow?: boolean;
  className?: string;
}

export function QRCode({
  value,
  size = 200,
  level = 'M',
  bgColor = '#ffffff',
  fgColor = '#0f172a',
  includeMargin = true,
  glow = true,
  className,
}: QRCodeProps) {
  if (!value) {
    return null;
  }

  const cardSize = Math.round(size + (includeMargin ? 48 : 24));

  return (
    <div
      className={cn('relative inline-flex items-center justify-center', className)}
      style={{ width: cardSize, height: cardSize }}
    >
      {glow && (
        <div
          aria-hidden="true"
          className="from-primary/20 absolute inset-0 rounded-2xl bg-gradient-to-br to-cyan-400/20 blur-xl"
        />
      )}
      <div className="relative flex aspect-square w-full items-center justify-center rounded-2xl border border-neutral-200 bg-white shadow-lg dark:border-slate-600">
        <QRCodeSVG
          value={value}
          size={size}
          level={level}
          bgColor={bgColor}
          fgColor={fgColor}
          includeMargin={includeMargin}
          role="img"
          aria-label="Connection QR code"
        />
      </div>
    </div>
  );
}
