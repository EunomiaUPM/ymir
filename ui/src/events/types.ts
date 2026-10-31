/** An event of the bus, as the events API returns it (ymir-events `EventEnvelope`). */
export interface EventEnvelope {
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

export interface EventCount {
  count: number;
}

export interface EventFeedParams {
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

export interface EventSubscription {
  id: string;
  topic_pattern: string;
  callback_address: string;
  active: boolean;
  /** Write-only: shows as `*****` when set. */
  secret?: string | null;
  retry_limit?: number | null;
  created_at: string;
}

export interface CreateSubscriptionCommand {
  topic_pattern: string;
  callback_address: string;
  secret?: string;
  retry_limit?: number;
}

export interface SubscriptionParams {
  active?: boolean;
  limit?: number;
  page?: number;
  cursor?: string;
  sort?: string;
}

export type DeadLetterStatus = "Unresolved" | "Replayed" | "Purged";

export interface DeadLetterRecord {
  id: string;
  topic: string;
  status: DeadLetterStatus;
  error_message: string;
  callback_address: string;
  attempts: number;
  failed_at: string;
  payload: unknown;
}

export interface DeadLetterParams {
  status?: DeadLetterStatus;
  limit?: number;
  page?: number;
  cursor?: string;
  sort?: string;
}

export interface ReplayAllResponse {
  replayed_count: number;
}

/** A page of the app the event is about; `params` fill the route's `$segments`. */
export interface EventLink {
  to: string;
  params?: Record<string, string>;
  /** The record's id as shown. */
  label: string;
}

/** What each app knows about its own topics: labels, feed areas and links to its pages. */
export interface EventTopicCatalog {
  /** Feed filters and subscription presets, as topic patterns. */
  areas: { label: string; pattern: string }[];
  /** Readable title, e.g. `heimdall:grant:create` → `Credential request created`. */
  describe(topic: string): string;
  /** Short area label of a topic. */
  area(topic: string): string;
  link(event: EventEnvelope): EventLink | undefined;
}

/** `customInstance`'s envelope; paged bodies come back as arrays carrying `items` and `total`. */
export interface ApiResponse<T> {
  status: number;
  data: T;
  headers: Headers;
}

export type Paged<T> = T[] & { items: T[]; total?: number; nextCursor?: string | null };
