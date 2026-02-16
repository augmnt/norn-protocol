import { cn } from "@/lib/utils";
import { Breadcrumb, type BreadcrumbItem } from "@/components/ui/breadcrumb";

interface PageContainerProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
  action?: React.ReactNode;
  breadcrumb?: BreadcrumbItem[];
}

export function PageContainer({
  title,
  description,
  children,
  className,
  action,
  breadcrumb,
}: PageContainerProps) {
  return (
    <div className={cn("mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8", className)}>
      {breadcrumb && breadcrumb.length > 0 && (
        <div className="mb-3">
          <Breadcrumb items={breadcrumb} />
        </div>
      )}
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
