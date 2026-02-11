"use client";

import { useWallet } from "@/hooks/use-wallet";
import { useSubscriptions } from "@/hooks/use-subscriptions";

export function SubscriptionsProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const { activeAddress, isUnlocked } = useWallet();

  // Only subscribe when wallet is unlocked
  useSubscriptions(isUnlocked ? activeAddress ?? undefined : undefined);

  return <>{children}</>;
}
