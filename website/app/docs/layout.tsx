import { DocsSidebar, MobileDocsSidebar } from "@/components/layout/docs-sidebar";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      <MobileDocsSidebar />
      <div className="flex gap-10">
        <div className="hidden md:block">
          <DocsSidebar />
        </div>
        <article className="min-w-0 flex-1 max-w-3xl">{children}</article>
      </div>
    </div>
  );
}
