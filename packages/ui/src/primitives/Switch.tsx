/**
 * Switch primitive built on @base-ui/react. Teal when checked; sized to match
 * the liquid-glass settings rows. Base UI exposes state via `data-checked` /
 * `data-unchecked`, so the track colour is driven by those attributes.
 */
import { Switch as BaseSwitch } from '@base-ui/react/switch';
import type { Ref } from 'react';

import { cn } from '../lib/cn';

export interface SwitchProps {
  checked?: boolean;
  defaultChecked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
  name?: string;
  id?: string;
  className?: string;
  ref?: Ref<HTMLButtonElement>;
}

export function Switch({
  checked,
  defaultChecked,
  onCheckedChange,
  disabled,
  name,
  id,
  className,
  ref,
}: SwitchProps) {
  return (
    <BaseSwitch.Root
      ref={ref}
      id={id}
      name={name}
      disabled={disabled}
      checked={checked}
      defaultChecked={defaultChecked}
      onCheckedChange={(value) => onCheckedChange?.(value)}
      className={cn(
        'relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors',
        'focus-visible:ring-primary-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'data-[checked]:bg-primary-600 data-[unchecked]:bg-neutral-300 dark:data-[unchecked]:bg-neutral-700',
        className
      )}
    >
      <BaseSwitch.Thumb
        className={cn(
          'pointer-events-none block h-4 w-4 rounded-full bg-white shadow-sm transition-transform duration-200 ease-out',
          'data-[checked]:translate-x-4 data-[unchecked]:translate-x-0.5'
        )}
      />
    </BaseSwitch.Root>
  );
}
