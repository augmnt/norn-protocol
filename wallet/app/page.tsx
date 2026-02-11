"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@/hooks/use-wallet";

export default function RootPage() {
  const router = useRouter();
  const { state } = useWallet();

  useEffect(() => {
    switch (state) {
      case "uninitialized":
        router.replace("/onboarding");
        break;
      case "locked":
        router.replace("/unlock");
        break;
      case "unlocked":
        router.replace("/dashboard");
        break;
    }
  }, [state, router]);

  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-muted border-t-foreground" />
    </div>
  );
}
