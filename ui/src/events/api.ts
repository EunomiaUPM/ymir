import { customInstance } from "../lib/http";
import type {
  ApiResponse,
  CreateSubscriptionCommand,
  DeadLetterParams,
  DeadLetterRecord,
  EventCount,
  EventEnvelope,
  EventFeedParams,
  EventSubscription,
  Paged,
  ReplayAllResponse,
  SubscriptionParams,
} from "./types";

const ROOT = "/events";

/** Query keys of the events API; all start with `/events`, like orval's path keys. */
export const EventsQueryKeys = {
  root: [ROOT] as const,
  feed: (params?: EventFeedParams) => [ROOT, ...(params ? [params] : [])] as const,
  unreadCount: () => [`${ROOT}/unread-count`] as const,
  subscriptions: (params?: SubscriptionParams) =>
    [`${ROOT}/subscriptions`, ...(params ? [params] : [])] as const,
  deadLetters: (params?: DeadLetterParams) =>
    [`${ROOT}/dlq`, ...(params ? [params] : [])] as const,
};

/** Calls of ymir-events' HTTP API, through the configured gateway. */
export const EventsApi = {
  feed: (params?: EventFeedParams) =>
    customInstance<ApiResponse<Paged<EventEnvelope>>>(ROOT, { params: { ...params } }),
  unreadCount: () => customInstance<ApiResponse<EventCount>>(`${ROOT}/unread-count`),
  markRead: (id: string) =>
    customInstance<ApiResponse<EventCount>>(`${ROOT}/${encodeURIComponent(id)}/read`, {
      method: "POST",
    }),
  markAllRead: () =>
    customInstance<ApiResponse<EventCount>>(`${ROOT}/read-all`, { method: "POST" }),

  subscriptions: (params?: SubscriptionParams) =>
    customInstance<ApiResponse<Paged<EventSubscription>>>(`${ROOT}/subscriptions`, {
      params: { ...params },
    }),
  createSubscription: (data: CreateSubscriptionCommand) =>
    customInstance<ApiResponse<EventSubscription>>(`${ROOT}/subscriptions`, {
      method: "POST",
      data,
    }),
  deleteSubscription: (id: string) =>
    customInstance<ApiResponse<unknown>>(`${ROOT}/subscriptions/${encodeURIComponent(id)}`, {
      method: "DELETE",
    }),

  deadLetters: (params?: DeadLetterParams) =>
    customInstance<ApiResponse<Paged<DeadLetterRecord>>>(`${ROOT}/dlq`, { params: { ...params } }),
  replayDeadLetter: (id: string) =>
    customInstance<ApiResponse<unknown>>(`${ROOT}/dlq/${encodeURIComponent(id)}/replay`, {
      method: "POST",
    }),
  replayAllDeadLetters: () =>
    customInstance<ApiResponse<ReplayAllResponse>>(`${ROOT}/dlq/replay-all`, { method: "POST" }),
  deleteDeadLetter: (id: string) =>
    customInstance<ApiResponse<unknown>>(`${ROOT}/dlq/${encodeURIComponent(id)}`, {
      method: "DELETE",
    }),
};
