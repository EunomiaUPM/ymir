import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { UseQueryOptions } from "@tanstack/react-query";
import { EventsApi, EventsQueryKeys } from "./api";
import type { DeadLetterParams, EventFeedParams, SubscriptionParams } from "./types";

type QueryExtras = Omit<UseQueryOptions<any, any, any, any>, "queryKey" | "queryFn">;

/** React Query hooks over `EventsApi`; mutations refresh every events query. */
export const EventsQueries = {
  useFeed: (params?: EventFeedParams, options?: QueryExtras) =>
    useQuery({
      queryKey: EventsQueryKeys.feed(params),
      queryFn: () => EventsApi.feed(params),
      ...options,
    }),
  useUnreadCount: () =>
    useQuery({ queryKey: EventsQueryKeys.unreadCount(), queryFn: () => EventsApi.unreadCount() }),
  useSubscriptions: (params?: SubscriptionParams, options?: QueryExtras) =>
    useQuery({
      queryKey: EventsQueryKeys.subscriptions(params),
      queryFn: () => EventsApi.subscriptions(params),
      ...options,
    }),
  useDeadLetters: (params?: DeadLetterParams, options?: QueryExtras) =>
    useQuery({
      queryKey: EventsQueryKeys.deadLetters(params),
      queryFn: () => EventsApi.deadLetters(params),
      ...options,
    }),

  /** A mutation that fails on an error status and refreshes the events queries on success. */
  useAction: <A, R extends { status: number }>(
    call: (arg: A) => Promise<R>,
    handlers?: { onSuccess?: (res: R) => void; onError?: (err: unknown) => void },
  ) => {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: async (arg: A) => {
        const res = await call(arg);
        if (res.status >= 400) throw new Error(EventsQueries.errorMessage(res));
        return res;
      },
      onSuccess: (res) => {
        void queryClient.invalidateQueries({
          predicate: (q) =>
            typeof q.queryKey[0] === "string" && q.queryKey[0].startsWith(EventsQueryKeys.root[0]),
        });
        handlers?.onSuccess?.(res);
      },
      onError: handlers?.onError,
    });
  },

  /** The API's `message`, or the status. */
  errorMessage: (res: { status: number; data?: unknown }): string => {
    const body = res.data as { message?: unknown } | undefined;
    return typeof body?.message === "string" ? body.message : `HTTP ${res.status}`;
  },
};
