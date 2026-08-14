/**
 * Progress primitive built on @base-ui/react (spec 07 §2.2).
 *
 * Wraps Base UI's Progress with Tailwind-driven styling. Base UI handles the
 * ARIA progressbar role and `aria-valuenow/min/max` attributes; we provide
 * the visual Track + Indicator pair.
 */
import { Progress as BaseProgress } from '@base-ui/react/progress';

import { cn } from '../lib/cn';

export interface ProgressProps {
  /** Current progress value. `null` or non-finite renders an indeterminate bar. */
  value: number | null;
  max?: number;
  className?: string;
  /** Accessible label describing what is progressing. */
  'aria-label'?: string;
}

export function Progress({ value, max = 100, className, 'aria-label': ariaLabel }: ProgressProps) {
  return (
    <BaseProgress.Root
      value={value}
      max={max}
      aria-label={ariaLabel}
      className={cn(
        'h-2 w-full overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-800',
        className
      )}
    >
      <BaseProgress.Track className="h-full w-full">
        <BaseProgress.Indicator
          className={cn(
            'bg-primary-600 h-full rounded-full transition-[width] duration-300 ease-in-out',
            'data-[status]:bg-primary-600'
          )}
        />
      </BaseProgress.Track>
    </BaseProgress.Root>
  );
}
