export type Visibility = "Private" | "Anonymous" | "Public";

export type EventCategory =
  | "AuditLifecycle"
  | "UsageControl"
  | "Attestation"
  | "DataAccess"
  | "ResourceChange"
  | "Security"
  | "Diagnostic";

export type EventSeverity = "Debug" | "Info" | "Warn" | "Error" | "Critical";

export type EventOutcome = "Success" | "Denied" | "Failed" | "Pending";

export type DeliveryClass = "AuditDurable" | "TelemetryStream";

export type ActorType =
  | "HumanUser"
  | "PeerParticipant"
  | "AgentService"
  | "SystemWorker"
  | "Anonymous";

export type WebhookDeliveryFormat = "envelope" | "payload_only";

export interface EventSource {
  dataspace_id?: string | null;
  participant_id?: string | null;
  service: string;
  instance_id: string;
  crate_name: string;
}

export interface EventActor {
  actor_type: ActorType;
  id: string;
  participant_id?: string | null;
  role?: string | null;
}

export interface EntityRef {
  kind: string;
  id: string;
  parent_id?: string | null;
}

export interface FlowRefs {
  agreement_id?: string | null;
  transfer_id?: string | null;
  negotiation_id?: string | null;
  grant_id?: string | null;
  dataset_id?: string | null;
  peer_id?: string | null;
}

export interface EventCorrelation {
  correlation_id?: string | null;
  causation_id?: string | null;
  trace_context?: string | null;
  flow_refs: FlowRefs;
}

export interface EventAttestation {
  format: string;
  signer_did: string;
  signature: string;
}

export interface EventIntegrity {
  payload_digest: string;
  canonicalization: string;
  attestation?: EventAttestation | null;
}

export interface EventVisibilityOwner {
  user_id: string;
  role: string;
  visibility: Visibility;
}

/** An event of the bus, as the events API returns it (ymir-events `EventView` over `EventEnvelope` v2). */
export interface EventEnvelope {
  id: string;
  spec_version: string;
  timestamp: string;
  sequence?: number;
  topic: string;
  event_type: EventCategory;
  severity: EventSeverity;
  outcome?: EventOutcome;
  delivery_class: DeliveryClass;
  schema_version: number;
  payload_type: string;
  source: EventSource;
  actor: EventActor;
  subject?: EntityRef | null;
  correlation: EventCorrelation;
  integrity: EventIntegrity;
  visibility_owner: EventVisibilityOwner;
  payload: unknown;
  /** Null while unread. */
  read_at?: string | null;
}

export interface EventCount {
  count: number;
}

export interface EventFeedParams {
  topic?: string;
  dataspace_id?: string;
  participant_id?: string;
  service?: string;
  source_crate?: string;
  event_type?: EventCategory;
  severity?: EventSeverity;
  outcome?: EventOutcome;
  payload_type?: string;
  actor_id?: string;
  subject_kind?: string;
  subject_id?: string;
  correlation_id?: string;
  agreement_id?: string;
  transfer_id?: string;
  negotiation_id?: string;
  grant_id?: string;
  dataset_id?: string;
  peer_id?: string;
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
  user_id: string;
  role: string;
  visibility: Visibility;
  topic_pattern: string;
  callback_address: string;
  active: boolean;
  /** Write-only: shows as `*****` when set. */
  secret?: string | null;
  retry_limit?: number | null;
  delivery_format: WebhookDeliveryFormat;
  created_at: string;
  updated_at?: string | null;
  expiration_time?: string | null;
}

export interface CreateSubscriptionCommand {
  visibility?: Visibility;
  topic_pattern: string;
  callback_address: string;
  secret?: string | null;
  retry_limit?: number | null;
  delivery_format?: WebhookDeliveryFormat;
  expiration_time?: string | null;
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
