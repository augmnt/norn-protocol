import Link from "next/link";
import { Button } from "@/components/ui/button";
import { PageContainer } from "@/components/ui/page-container";

export default function NotFound() {
  return (
    <PageContainer>
      <div className="flex flex-col items-center justify-center py-24 text-center">
        <h1 className="text-display mb-2">404</h1>
        <p className="text-muted-foreground mb-6">Page not found</p>
        <Button asChild variant="outline">
          <Link href="/">Back to Dashboard</Link>
        </Button>
      </div>
    </PageContainer>
  );
}
