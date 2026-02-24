"use client";

import { useEffect } from "react";

const SW_VERSION = 4;

export function ServiceWorkerRegister() {
  useEffect(() => {
    if (!("serviceWorker" in navigator)) return;

    const storedVersion = localStorage.getItem("sw-version");
    const currentVersion = String(SW_VERSION);

    if (storedVersion !== currentVersion) {
      // Version mismatch — unregister old SW, clear all caches, then re-register
      navigator.serviceWorker.getRegistrations().then((registrations) => {
        const unregisterAll = registrations.map((r) => r.unregister());
        return Promise.all(unregisterAll);
      }).then(() => {
        return caches.keys();
      }).then((keys) => {
        return Promise.all(keys.map((key) => caches.delete(key)));
      }).then(() => {
        localStorage.setItem("sw-version", currentVersion);
        // Re-register fresh SW
        navigator.serviceWorker.register("/sw.js").catch(() => {});
      }).catch(() => {});
    } else {
      navigator.serviceWorker.register("/sw.js").catch(() => {});
    }
  }, []);

  return null;
}
