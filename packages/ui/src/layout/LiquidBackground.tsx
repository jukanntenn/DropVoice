/**
 * LiquidBackground — the fixed decorative layer for the liquid-glass theme:
 * two soft blurred colour orbs floating over the page gradient (authored on
 * `body` in effects.css). Render once at the root of each app, behind content.
 */
import { cn } from '../lib/cn';

export interface LiquidBackgroundProps {
  className?: string;
}

export function LiquidBackground({ className }: LiquidBackgroundProps) {
  return (
    <div
      aria-hidden="true"
      className={cn('pointer-events-none fixed inset-0 -z-10 overflow-hidden', className)}
    >
      <div className="bg-primary/10 absolute -right-40 -top-40 h-80 w-80 rounded-full blur-3xl" />
      <div className="absolute -bottom-40 -left-40 h-80 w-80 rounded-full bg-cyan-400/10 blur-3xl dark:bg-cyan-500/5" />
    </div>
  );
}
