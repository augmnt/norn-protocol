import { create } from "zustand";
import type { Route, SendParams } from "@/types";

interface RouteEntry {
  route: Route;
  params?: Record<string, unknown>;
}

interface NavigationState {
  currentRoute: Route;
  params: Record<string, unknown>;
  history: RouteEntry[];
  navigate: (route: Route, params?: Record<string, unknown>) => void;
  goBack: () => void;
  reset: (route: Route) => void;
  getSendParams: () => SendParams | null;
}

export const useNavigationStore = create<NavigationState>((set, get) => ({
  currentRoute: "welcome",
  params: {},
  history: [],

  navigate: (route, params = {}) => {
    const { currentRoute, params: currentParams, history } = get();
    set({
      currentRoute: route,
      params,
      history: [...history, { route: currentRoute, params: currentParams }],
    });
  },

  goBack: () => {
    const { history } = get();
    if (history.length === 0) return;
    const prev = history[history.length - 1];
    set({
      currentRoute: prev.route,
      params: prev.params ?? {},
      history: history.slice(0, -1),
    });
  },

  reset: (route) => {
    set({ currentRoute: route, params: {}, history: [] });
  },

  getSendParams: () => {
    const { params } = get();
    if (params.to && params.amount) {
      return params as unknown as SendParams;
    }
    return null;
  },
}));
