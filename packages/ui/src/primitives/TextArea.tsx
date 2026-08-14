/**
 * TextArea primitive — native `<textarea>` styled with the liquid-glass language.
 * @base-ui/react has no textarea primitive, so this wraps the native element.
 */
import { cva } from 'class-variance-authority';
import type { Ref, TextareaHTMLAttributes } from 'react';

import { cn } from '../lib/cn';

const textareaVariants = cva(
  'w-full resize-y rounded-2xl border border-white/60 bg-white/50 px-4 py-3 text-[15px] leading-relaxed text-neutral-900 transition-all duration-200 placeholder:text-neutral-400 focus:border-primary/50 focus:bg-white/70 focus:outline-none focus:ring-4 focus:ring-primary/10 disabled:cursor-not-allowed disabled:opacity-50 dark:border-white/10 dark:bg-slate-800/50 dark:text-slate-100 dark:placeholder:text-slate-400 dark:focus:bg-slate-800/70 dark:focus:ring-primary/20'
);

export interface TextAreaProps extends Omit<TextareaHTMLAttributes<HTMLTextAreaElement>, 'size'> {
  className?: string;
  ref?: Ref<HTMLTextAreaElement>;
}

export function TextArea({ className, ref, ...props }: TextAreaProps) {
  return <textarea ref={ref} className={cn(textareaVariants(), className)} {...props} />;
}
