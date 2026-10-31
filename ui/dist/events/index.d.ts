import * as _tanstack_react_query from '@tanstack/react-query';
import { UseQueryOptions, QueryClient } from '@tanstack/react-query';
import * as react_jsx_runtime from 'react/jsx-runtime';
import { ReactNode } from 'react';

/** An event of the bus, as the events API returns it (ymir-events `EventEnvelope`). */
interface EventEnvelope {
    id: string;
    user_id: string;
    role: string;
    visibility: "Private" | "Anonymous" | "Public";
    topic: string;
    source_crate: string;
    schema_version: number;
    timestamp: string;
    correlation_id?: string;
    /** W3C `traceparent` of the publishing span. */
    trace_context?: string;
    payload: unknown;
    /** Null while unread. */
    read_at?: string | null;
}
interface EventCount {
    count: number;
}
interface EventFeedParams {
    topic?: string;
    source_crate?: string;
    read?: boolean;
    created_after?: string;
    created_before?: string;
    limit?: number;
    page?: number;
    cursor?: string;
    sort?: string;
}
interface EventSubscription {
    id: string;
    topic_pattern: string;
    callback_address: string;
    active: boolean;
    /** Write-only: shows as `*****` when set. */
    secret?: string | null;
    retry_limit?: number | null;
    created_at: string;
}
interface CreateSubscriptionCommand {
    topic_pattern: string;
    callback_address: string;
    secret?: string;
    retry_limit?: number;
}
interface SubscriptionParams {
    active?: boolean;
    limit?: number;
    page?: number;
    cursor?: string;
    sort?: string;
}
type DeadLetterStatus = "Unresolved" | "Replayed" | "Purged";
interface DeadLetterRecord {
    id: string;
    topic: string;
    status: DeadLetterStatus;
    error_message: string;
    callback_address: string;
    attempts: number;
    failed_at: string;
    payload: unknown;
}
interface DeadLetterParams {
    status?: DeadLetterStatus;
    limit?: number;
    page?: number;
    cursor?: string;
    sort?: string;
}
interface ReplayAllResponse {
    replayed_count: number;
}
/** A page of the app the event is about; `params` fill the route's `$segments`. */
interface EventLink {
    to: string;
    params?: Record<string, string>;
    /** The record's id as shown. */
    label: string;
}
/** What each app knows about its own topics: labels, feed areas and links to its pages. */
interface EventTopicCatalog {
    /** Feed filters and subscription presets, as topic patterns. */
    areas: {
        label: string;
        pattern: string;
    }[];
    /** Readable title, e.g. `heimdall:grant:create` → `Credential request created`. */
    describe(topic: string): string;
    /** Short area label of a topic. */
    area(topic: string): string;
    link(event: EventEnvelope): EventLink | undefined;
}
/** `customInstance`'s envelope; paged bodies come back as arrays carrying `items` and `total`. */
interface ApiResponse<T> {
    status: number;
    data: T;
    headers: Headers;
}
type Paged<T> = T[] & {
    items: T[];
    total?: number;
    nextCursor?: string | null;
};

/** Query keys of the events API; all start with `/events`, like orval's path keys. */
declare const EventsQueryKeys: {
    root: readonly ["/events"];
    feed: (params?: EventFeedParams) => readonly ["/events", ...EventFeedParams[]];
    unreadCount: () => readonly ["/events/unread-count"];
    subscriptions: (params?: SubscriptionParams) => readonly ["/events/subscriptions", ...SubscriptionParams[]];
    deadLetters: (params?: DeadLetterParams) => readonly ["/events/dlq", ...DeadLetterParams[]];
};
/** Calls of ymir-events' HTTP API, through the configured gateway. */
declare const EventsApi: {
    feed: (params?: EventFeedParams) => Promise<ApiResponse<Paged<EventEnvelope>>>;
    unreadCount: () => Promise<ApiResponse<EventCount>>;
    markRead: (id: string) => Promise<ApiResponse<EventCount>>;
    markAllRead: () => Promise<ApiResponse<EventCount>>;
    subscriptions: (params?: SubscriptionParams) => Promise<ApiResponse<Paged<EventSubscription>>>;
    createSubscription: (data: CreateSubscriptionCommand) => Promise<ApiResponse<EventSubscription>>;
    deleteSubscription: (id: string) => Promise<ApiResponse<unknown>>;
    deadLetters: (params?: DeadLetterParams) => Promise<ApiResponse<Paged<DeadLetterRecord>>>;
    replayDeadLetter: (id: string) => Promise<ApiResponse<unknown>>;
    replayAllDeadLetters: () => Promise<ApiResponse<ReplayAllResponse>>;
    deleteDeadLetter: (id: string) => Promise<ApiResponse<unknown>>;
};

type QueryExtras = Omit<UseQueryOptions<any, any, any, any>, "queryKey" | "queryFn">;
/** React Query hooks over `EventsApi`; mutations refresh every events query. */
declare const EventsQueries: {
    useFeed: (params?: EventFeedParams, options?: QueryExtras) => _tanstack_react_query.UseQueryResult<any, any>;
    useUnreadCount: () => _tanstack_react_query.UseQueryResult<ApiResponse<EventCount>, Error>;
    useSubscriptions: (params?: SubscriptionParams, options?: QueryExtras) => _tanstack_react_query.UseQueryResult<any, any>;
    useDeadLetters: (params?: DeadLetterParams, options?: QueryExtras) => _tanstack_react_query.UseQueryResult<any, any>;
    /** A mutation that fails on an error status and refreshes the events queries on success. */
    useAction: <A, R extends {
        status: number;
    }>(call: (arg: A) => Promise<R>, handlers?: {
        onSuccess?: (res: R) => void;
        onError?: (err: unknown) => void;
    }) => _tanstack_react_query.UseMutationResult<R, unknown, A, unknown>;
    /** The API's `message`, or the status. */
    errorMessage: (res: {
        status: number;
        data?: unknown;
    }) => string;
};

type Listener = (event: EventEnvelope) => void;
/** How the app reaches the bus: an SSE endpoint or a WebSocket (ds-agent's BFF `/live`). */
type LiveTransport = "sse" | "websocket";
/** Query key prefixes (orval keys are request paths) refreshed by each topic prefix. */
type LiveInvalidations = [topicPrefix: string, keyPrefixes: string[]][];
interface LiveContextType {
    connected: boolean;
    /** Latest events, newest first. */
    events: EventEnvelope[];
    /** Calls `listener` for every event until the returned function runs. */
    subscribe: (listener: Listener) => () => void;
}
interface LiveProviderProps {
    transport: LiveTransport;
    /** A stable function, resolved on mount so it reads the configured gateway. */
    url: () => string;
    invalidations?: LiveInvalidations;
    children: ReactNode;
}
/** One live connection to the bus for the whole app; events refresh the queries they touch. */
declare const LiveProvider: {
    ({ transport, url, invalidations, children }: LiveProviderProps): react_jsx_runtime.JSX.Element;
    openSse(url: string, receive: (data: string) => void, setConnected: (c: boolean) => void): (() => void);
    openWebSocket(url: string, receive: (data: string) => void, setConnected: (c: boolean) => void): (() => void);
    prefixesFor(topic: string, invalidations: LiveInvalidations): string[];
    invalidate(queryClient: QueryClient, prefixes: string[]): void;
};
/** Live connection state and events; `subscribe` for a callback per event. */
declare const useLive: () => LiveContextType;

interface NotificationBellProps {
    topics: EventTopicCatalog;
    /** The app's event feed page. */
    feedTo: string;
}
/** Header bell: unread bus events, kept current by the live connection, marked read on open. */
declare const NotificationBell: ({ topics, feedTo }: NotificationBellProps) => react_jsx_runtime.JSX.Element;

interface EventFeedProps {
    topics: EventTopicCatalog;
}
/** The events the user sees, filterable by area, read state and period; a row opens its payload. */
declare const EventFeed: {
    ({ topics }: EventFeedProps): react_jsx_runtime.JSX.Element;
    RecordLink({ event, topics }: {
        event: EventEnvelope;
        topics: EventTopicCatalog;
    }): react_jsx_runtime.JSX.Element;
};

interface EventSubscriptionsProps {
    topics: EventTopicCatalog;
    /** Pattern the new-subscription dialog starts with, e.g. `heimdall:**`. */
    defaultPattern: string;
}
/** Webhook subscriptions of the bus: list, create with HMAC secret and retries, delete. */
declare const EventSubscriptions: ({ topics, defaultPattern }: EventSubscriptionsProps) => react_jsx_runtime.JSX.Element;

/** Deliveries that kept failing: inspect, replay one or all, or discard. */
declare const DeadLetters: () => react_jsx_runtime.JSX.Element;

interface EventsLayoutProps {
    /** Route the three tabs hang from, e.g. `/events`. */
    basePath: string;
    description: string;
    /** The current tab, usually the router's `<Outlet />`. */
    children: ReactNode;
}
/** Events page with its Feed, Webhook subscriptions and Dead letters tabs. */
declare const EventsLayout: ({ basePath, description, children }: EventsLayoutProps) => react_jsx_runtime.JSX.Element;

export { type ApiResponse, type CreateSubscriptionCommand, type DeadLetterParams, type DeadLetterRecord, type DeadLetterStatus, DeadLetters, type EventCount, type EventEnvelope, EventFeed, type EventFeedParams, type EventFeedProps, type EventLink, type EventSubscription, EventSubscriptions, type EventSubscriptionsProps, type EventTopicCatalog, EventsApi, EventsLayout, type EventsLayoutProps, EventsQueries, EventsQueryKeys, type LiveContextType, type LiveInvalidations, LiveProvider, type LiveProviderProps, type LiveTransport, NotificationBell, type NotificationBellProps, type Paged, type ReplayAllResponse, type SubscriptionParams, useLive };
