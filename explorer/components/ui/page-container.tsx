import { cn } from "@/lib/utils";

interface PageContainerProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
  action?: React.ReactNode;
}

export function PageContainer({
  title,
  description,
  children,
  className,
  action,
}: PageContainerProps) {
  return (
    <div className={cn("mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8", className)}>
      {(title || action) && (
        <div className="mb-6 flex items-center justify-between">
          <div>
            {title && (
              <h1 className="text-heading font-semibold">{title}</h1>
            )}
            {description && (
              <p className="mt-1 text-sm text-muted-foreground">{description}</p>
            )}
          </div>
          {action}
        </div>
      )}
      {children}
    </div>
  );
}
