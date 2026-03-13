import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { HTMLAttributes } from 'react';

export function cn(...inputs: Array<string | undefined | false>) {
  return twMerge(clsx(inputs));
}

export function Card(props: HTMLAttributes<HTMLDivElement>) {
  return <div {...props} className={cn('rounded-xl border border-slate-800 bg-card/80 p-4 shadow-lg', props.className)} />;
}
