"use client";

import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { AddressDisplay } from "@/components/ui/address-display";
import { Button } from "@/components/ui/button";
import { useFavoritesStore } from "@/stores/favorites-store";
import { Star, X } from "lucide-react";

export function FavoritesBar() {
  const favorites = useFavoritesStore((s) => s.favorites);
  const removeFavorite = useFavoritesStore((s) => s.removeFavorite);

  if (favorites.length === 0) return null;

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center gap-2">
          <Star className="h-4 w-4 fill-yellow-500 text-yellow-500" />
          <CardTitle className="text-sm font-medium">Watchlist</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex flex-wrap gap-2">
          {favorites.map((fav) => (
            <div
              key={fav.address}
              className="inline-flex items-center gap-1 rounded-md border px-2 py-1"
            >
              <Link
                href={`/address/${fav.address}`}
                className="text-norn hover:underline"
              >
                <AddressDisplay
                  address={fav.address}
                  link={false}
                  copy={false}
                />
              </Link>
              <button
                onClick={() => removeFavorite(fav.address)}
                className="ml-1 text-muted-foreground hover:text-foreground"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
