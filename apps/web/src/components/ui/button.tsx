import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 font-sans font-semibold tracking-tight rounded-sm transition-[transform,background-color,box-shadow,color,opacity] duration-150 ease-out active:scale-[0.96] disabled:opacity-50 disabled:pointer-events-none",
  {
    variants: {
      variant: {
        primary: "bg-accent text-accent-fg hover:opacity-90",
        ghost: "bg-transparent text-fg shadow-hairline hover:bg-raised",
        quiet: "bg-raised text-fg hover:bg-surface",
        kill: "bg-refuse text-refuse-fg hover:opacity-90",
        invert: "bg-bg text-fg hover:opacity-90",
      },
      size: {
        default: "min-h-11 px-5 text-sm",
        sm: "min-h-10 px-3 text-xs",
        lg: "min-h-12 px-6 text-sm",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "default",
    },
  },
);

export function Button({
  className,
  variant,
  size,
  asChild,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof buttonVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp className={cn(buttonVariants({ variant, size }), className)} {...props} />
  );
}
