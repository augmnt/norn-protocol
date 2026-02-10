import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function NotFound() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center py-24">
      <h1 className="font-mono text-6xl font-bold text-muted-foreground">404</h1>
      <p className="mt-4 text-lg text-muted-foreground">Page not found</p>
      <Button asChild variant="outline" className="mt-8">
        <Link href="/">Go home</Link>
      </Button>
    </div>
  );
}
