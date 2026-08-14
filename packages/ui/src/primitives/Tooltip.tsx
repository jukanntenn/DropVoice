/**
 * Tooltip primitive built on @base-ui/react.
 *
 * Uses an inverted colour scheme — dark tooltip on the light theme, white tooltip
 * on the dark theme — matching the original DropVoice liquid-glass header.
 */
import { Tooltip as BaseTooltip } from '@base-ui/react/tooltip';
import type { ReactElement, ReactNode } from 'react';

import { cn } from '../lib/cn';

export interface TooltipProps {
  content: ReactNode;
  side?: 'top' | 'bottom' | 'left' | 'right';
  /** A single trigger element (rendered via Base UI's `render` prop). */
  children: ReactElement;
  className?: string;
}

export function Tooltip({ content, side = 'top', children, className }: TooltipProps) {
  return (
    <BaseTooltip.Provider delay={200} closeDelay={0}>
      <BaseTooltip.Root>
        <BaseTooltip.Trigger render={children} />
        <BaseTooltip.Portal>
          <BaseTooltip.Positioner side={side} sideOffset={6} className={cn('z-50', className)}>
            <BaseTooltip.Popup className="animate-scale-in rounded-lg bg-slate-900 px-3 py-1.5 text-xs font-medium text-white shadow-md dark:bg-white dark:text-slate-900">
              {content}
            </BaseTooltip.Popup>
          </BaseTooltip.Positioner>
        </BaseTooltip.Portal>
      </BaseTooltip.Root>
    </BaseTooltip.Provider>
  );
}
