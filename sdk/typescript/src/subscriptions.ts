import type {
  BlockInfo,
  TransferEvent,
  TokenEvent,
  LoomExecutionEvent,
  PendingTransactionEvent,
} from "./types.js";

/** A subscription that can be unsubscribed. */
export class Subscription {
  private ws: WebSocket;
  private closed = false;

  constructor(ws: WebSocket) {
    this.ws = ws;
  }

  /** Close the subscription and WebSocket connection. */
  unsubscribe(): void {
    if (!this.closed) {
      this.closed = true;
      this.ws.close();
    }
  }

  /** Whether the subscription has been closed. */
  get isActive(): boolean {
    return !this.closed && this.ws.readyState === WebSocket.OPEN;
  }
}

/** Options for WebSocket subscriptions. */
export interface SubscribeOptions {
  /** WebSocket URL (e.g. "ws://localhost:9944"). */
  url: string;
  /** Called when the connection is established. */
  onOpen?: () => void;
  /** Called when the connection is closed. */
  onClose?: () => void;
  /** Called when an error occurs. */
  onError?: (error: Event) => void;
}

/** Helper to subscribe to a JSON-RPC subscription. */
function subscribe<T>(
  options: SubscribeOptions,
  method: string,
  params: unknown[],
  onMessage: (data: T) => void,
): Subscription {
  const ws = new WebSocket(options.url);
  let subscriptionId: string | null = null;

  ws.onopen = () => {
    const req = {
      jsonrpc: "2.0",
      method,
      params,
      id: 1,
    };
    ws.send(JSON.stringify(req));
    options.onOpen?.();
  };

  ws.onmessage = (event: MessageEvent) => {
    try {
      const msg = JSON.parse(String(event.data));
      // First response is the subscription ID.
      if (msg.id === 1 && msg.result) {
        subscriptionId = msg.result;
        return;
      }
      // Subsequent messages are subscription notifications.
      if (msg.method && msg.params?.subscription === subscriptionId) {
        onMessage(msg.params.result as T);
      }
    } catch {
      // Ignore parse errors.
    }
  };

  ws.onclose = () => {
    options.onClose?.();
  };

  ws.onerror = (error: Event) => {
    options.onError?.(error);
  };

  return new Subscription(ws);
}

/** Subscribe to new blocks. */
export function subscribeNewBlocks(
  options: SubscribeOptions,
  onBlock: (block: BlockInfo) => void,
): Subscription {
  return subscribe(options, "norn_subscribeNewBlocks", [], onBlock);
}

/** Subscribe to transfer events, optionally filtered by address. */
export function subscribeTransfers(
  options: SubscribeOptions,
  onTransfer: (event: TransferEvent) => void,
  addressFilter?: string,
): Subscription {
  const params = addressFilter ? [addressFilter] : [];
  return subscribe(options, "norn_subscribeTransfers", params, onTransfer);
}

/** Subscribe to token events, optionally filtered by token ID. */
export function subscribeTokenEvents(
  options: SubscribeOptions,
  onEvent: (event: TokenEvent) => void,
  tokenIdFilter?: string,
): Subscription {
  const params = tokenIdFilter ? [tokenIdFilter] : [];
  return subscribe(options, "norn_subscribeTokenEvents", params, onEvent);
}

/** Subscribe to loom execution events, optionally filtered by loom ID. */
export function subscribeLoomEvents(
  options: SubscribeOptions,
  onEvent: (event: LoomExecutionEvent) => void,
  loomIdFilter?: string,
): Subscription {
  const params = loomIdFilter ? [loomIdFilter] : [];
  return subscribe(options, "norn_subscribeLoomEvents", params, onEvent);
}

/** Subscribe to pending transaction events. */
export function subscribePendingTransactions(
  options: SubscribeOptions,
  onEvent: (event: PendingTransactionEvent) => void,
): Subscription {
  return subscribe(options, "norn_subscribePendingTransactions", [], onEvent);
}
