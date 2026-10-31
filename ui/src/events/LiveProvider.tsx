import React, {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { QueryClient, useQueryClient } from "@tanstack/react-query";
import { EventsQueryKeys } from "./api";
import type { EventEnvelope } from "./types";

type Listener = (event: EventEnvelope) => void;

/** How the app reaches the bus: an SSE endpoint or a WebSocket (ds-agent's BFF `/live`). */
export type LiveTransport = "sse" | "websocket";

/** Query key prefixes (orval keys are request paths) refreshed by each topic prefix. */
export type LiveInvalidations = [topicPrefix: string, keyPrefixes: string[]][];

export interface LiveContextType {
  connected: boolean;
  /** Latest events, newest first. */
  events: EventEnvelope[];
  /** Calls `listener` for every event until the returned function runs. */
  subscribe: (listener: Listener) => () => void;
}

export interface LiveProviderProps {
  transport: LiveTransport;
  /** A stable function, resolved on mount so it reads the configured gateway. */
  url: () => string;
  invalidations?: LiveInvalidations;
  children: ReactNode;
}

const LiveContext = createContext<LiveContextType | null>(null);
const BUFFER = 100;
const FLUSH_MS = 250;

/** One live connection to the bus for the whole app; events refresh the queries they touch. */
export const LiveProvider = ({ transport, url, invalidations = [], children }: LiveProviderProps) => {
  const queryClient = useQueryClient();
  const [connected, setConnected] = useState(false);
  const [events, setEvents] = useState<EventEnvelope[]>([]);
  const listeners = useRef(new Set<Listener>());
  // Read at use time so a new map from the app does not reopen the connection.
  const invalidationsRef = useRef(invalidations);
  invalidationsRef.current = invalidations;

  useEffect(() => {
    const pending = new Set<string>();
    let flush: ReturnType<typeof setTimeout> | undefined;

    // Bursts of events (a whole flow) collapse into one refresh per key prefix.
    const schedule = (topic: string) => {
      LiveProvider.prefixesFor(topic, invalidationsRef.current).forEach((p) => pending.add(p));
      flush ??= setTimeout(() => {
        flush = undefined;
        LiveProvider.invalidate(queryClient, [...pending]);
        pending.clear();
      }, FLUSH_MS);
    };

    const receive = (data: string) => {
      let envelope: EventEnvelope;
      try {
        envelope = JSON.parse(data);
      } catch {
        return;
      }
      setEvents((prev) => [envelope, ...prev].slice(0, BUFFER));
      listeners.current.forEach((l) => l(envelope));
      schedule(String(envelope.topic));
    };

    const close =
      transport === "sse"
        ? LiveProvider.openSse(url(), receive, setConnected)
        : LiveProvider.openWebSocket(url(), receive, setConnected);

    return () => {
      clearTimeout(flush);
      close();
    };
  }, [queryClient, transport, url]);

  const value = useMemo<LiveContextType>(
    () => ({
      connected,
      events,
      subscribe: (listener) => {
        listeners.current.add(listener);
        return () => listeners.current.delete(listener);
      },
    }),
    [connected, events],
  );

  return <LiveContext.Provider value={value}>{children}</LiveContext.Provider>;
};

/** EventSource reconnects on its own after a drop. */
LiveProvider.openSse = (
  url: string,
  receive: (data: string) => void,
  setConnected: (c: boolean) => void,
): (() => void) => {
  const source = new EventSource(url, { withCredentials: true });
  source.onopen = () => setConnected(true);
  source.onerror = () => setConnected(false);
  source.onmessage = (msg) => receive(msg.data);
  return () => source.close();
};

/** WebSocket with capped exponential backoff on reconnect. */
LiveProvider.openWebSocket = (
  url: string,
  receive: (data: string) => void,
  setConnected: (c: boolean) => void,
): (() => void) => {
  let socket: WebSocket | null = null;
  let retry: ReturnType<typeof setTimeout> | undefined;
  let attempts = 0;
  let stopped = false;
  const connect = () => {
    socket = new WebSocket(url);
    socket.onopen = () => {
      attempts = 0;
      setConnected(true);
    };
    socket.onmessage = (msg) => receive(String(msg.data));
    socket.onclose = () => {
      setConnected(false);
      if (stopped) return;
      retry = setTimeout(connect, Math.min(30_000, 1_000 * 2 ** attempts++));
    };
  };
  connect();
  return () => {
    stopped = true;
    clearTimeout(retry);
    socket?.close();
  };
};

/** Every event refreshes the event queries, plus the keys of its own area. */
LiveProvider.prefixesFor = (topic: string, invalidations: LiveInvalidations): string[] => [
  EventsQueryKeys.root[0],
  ...invalidations.filter(([t]) => topic.startsWith(t)).flatMap(([, keys]) => keys),
];

LiveProvider.invalidate = (queryClient: QueryClient, prefixes: string[]) => {
  if (prefixes.length === 0) return;
  queryClient.invalidateQueries({
    predicate: (q) => {
      const head = q.queryKey[0];
      return typeof head === "string" && prefixes.some((p) => head.startsWith(p));
    },
  });
};

/** Live connection state and events; `subscribe` for a callback per event. */
export const useLive = (): LiveContextType => {
  const ctx = useContext(LiveContext);
  if (!ctx) throw new Error("useLive must be used inside LiveProvider");
  return ctx;
};
