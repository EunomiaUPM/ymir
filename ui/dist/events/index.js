import {
  Popover,
  PopoverContent,
  PopoverTrigger,
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle
} from "../chunk-A6ZPWGMH.js";
import {
  JsonTable
} from "../chunk-VTWMQZBI.js";
import {
  Badge,
  Button,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FieldGrid,
  FormatDate,
  Input,
  PageHeader,
  PageLayout,
  PageSection,
  RelativeTime,
  RouteTabs,
  Skeleton,
  customInstance,
  useTableQueryParams
} from "../chunk-CZL7FYB6.js";
import {
  Identifier,
  cn
} from "../chunk-LDYPVTBN.js";

// src/events/api.ts
var ROOT = "/events";
var EventsQueryKeys = {
  root: [ROOT],
  feed: (params) => [ROOT, ...params ? [params] : []],
  unreadCount: () => [`${ROOT}/unread-count`],
  subscriptions: (params) => [`${ROOT}/subscriptions`, ...params ? [params] : []],
  deadLetters: (params) => [`${ROOT}/dlq`, ...params ? [params] : []]
};
var EventsApi = {
  feed: (params) => customInstance(ROOT, { params: { ...params } }),
  unreadCount: () => customInstance(`${ROOT}/unread-count`),
  markRead: (id) => customInstance(`${ROOT}/${encodeURIComponent(id)}/read`, {
    method: "POST"
  }),
  markAllRead: () => customInstance(`${ROOT}/read-all`, { method: "POST" }),
  subscriptions: (params) => customInstance(`${ROOT}/subscriptions`, {
    params: { ...params }
  }),
  createSubscription: (data) => customInstance(`${ROOT}/subscriptions`, {
    method: "POST",
    data
  }),
  deleteSubscription: (id) => customInstance(`${ROOT}/subscriptions/${encodeURIComponent(id)}`, {
    method: "DELETE"
  }),
  deadLetters: (params) => customInstance(`${ROOT}/dlq`, { params: { ...params } }),
  replayDeadLetter: (id) => customInstance(`${ROOT}/dlq/${encodeURIComponent(id)}/replay`, {
    method: "POST"
  }),
  replayAllDeadLetters: () => customInstance(`${ROOT}/dlq/replay-all`, { method: "POST" }),
  deleteDeadLetter: (id) => customInstance(`${ROOT}/dlq/${encodeURIComponent(id)}`, {
    method: "DELETE"
  })
};

// src/events/queries.ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
var EventsQueries = {
  useFeed: (params, options) => useQuery({
    queryKey: EventsQueryKeys.feed(params),
    queryFn: () => EventsApi.feed(params),
    ...options
  }),
  useUnreadCount: () => useQuery({ queryKey: EventsQueryKeys.unreadCount(), queryFn: () => EventsApi.unreadCount() }),
  useSubscriptions: (params, options) => useQuery({
    queryKey: EventsQueryKeys.subscriptions(params),
    queryFn: () => EventsApi.subscriptions(params),
    ...options
  }),
  useDeadLetters: (params, options) => useQuery({
    queryKey: EventsQueryKeys.deadLetters(params),
    queryFn: () => EventsApi.deadLetters(params),
    ...options
  }),
  /** A mutation that fails on an error status and refreshes the events queries on success. */
  useAction: (call, handlers) => {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: async (arg) => {
        const res = await call(arg);
        if (res.status >= 400) throw new Error(EventsQueries.errorMessage(res));
        return res;
      },
      onSuccess: (res) => {
        void queryClient.invalidateQueries({
          predicate: (q) => typeof q.queryKey[0] === "string" && q.queryKey[0].startsWith(EventsQueryKeys.root[0])
        });
        handlers?.onSuccess?.(res);
      },
      onError: handlers?.onError
    });
  },
  /** The API's `message`, or the status. */
  errorMessage: (res) => {
    const body = res.data;
    return typeof body?.message === "string" ? body.message : `HTTP ${res.status}`;
  }
};

// src/events/LiveProvider.tsx
import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState
} from "react";
import { useQueryClient as useQueryClient2 } from "@tanstack/react-query";
import { jsx } from "react/jsx-runtime";
var LiveContext = createContext(null);
var BUFFER = 100;
var FLUSH_MS = 250;
var LiveProvider = ({ transport, url, invalidations = [], children }) => {
  const queryClient = useQueryClient2();
  const [connected, setConnected] = useState(false);
  const [events, setEvents] = useState([]);
  const listeners = useRef(/* @__PURE__ */ new Set());
  const invalidationsRef = useRef(invalidations);
  invalidationsRef.current = invalidations;
  useEffect(() => {
    const pending = /* @__PURE__ */ new Set();
    let flush;
    const schedule = (topic) => {
      LiveProvider.prefixesFor(topic, invalidationsRef.current).forEach((p) => pending.add(p));
      flush ?? (flush = setTimeout(() => {
        flush = void 0;
        LiveProvider.invalidate(queryClient, [...pending]);
        pending.clear();
      }, FLUSH_MS));
    };
    const receive = (data) => {
      let envelope;
      try {
        envelope = JSON.parse(data);
      } catch {
        return;
      }
      setEvents((prev) => [envelope, ...prev].slice(0, BUFFER));
      listeners.current.forEach((l) => l(envelope));
      schedule(String(envelope.topic));
    };
    const close = transport === "sse" ? LiveProvider.openSse(url(), receive, setConnected) : LiveProvider.openWebSocket(url(), receive, setConnected);
    return () => {
      clearTimeout(flush);
      close();
    };
  }, [queryClient, transport, url]);
  const value = useMemo(
    () => ({
      connected,
      events,
      subscribe: (listener) => {
        listeners.current.add(listener);
        return () => listeners.current.delete(listener);
      }
    }),
    [connected, events]
  );
  return /* @__PURE__ */ jsx(LiveContext.Provider, { value, children });
};
LiveProvider.openSse = (url, receive, setConnected) => {
  const source = new EventSource(url, { withCredentials: true });
  source.onopen = () => setConnected(true);
  source.onerror = () => setConnected(false);
  source.onmessage = (msg) => receive(msg.data);
  return () => source.close();
};
LiveProvider.openWebSocket = (url, receive, setConnected) => {
  let socket = null;
  let retry;
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
      retry = setTimeout(connect, Math.min(3e4, 1e3 * 2 ** attempts++));
    };
  };
  connect();
  return () => {
    stopped = true;
    clearTimeout(retry);
    socket?.close();
  };
};
LiveProvider.prefixesFor = (topic, invalidations) => [
  EventsQueryKeys.root[0],
  ...invalidations.filter(([t]) => topic.startsWith(t)).flatMap(([, keys]) => keys)
];
LiveProvider.invalidate = (queryClient, prefixes) => {
  if (prefixes.length === 0) return;
  queryClient.invalidateQueries({
    predicate: (q) => {
      const head = q.queryKey[0];
      return typeof head === "string" && prefixes.some((p) => head.startsWith(p));
    }
  });
};
var useLive = () => {
  const ctx = useContext(LiveContext);
  if (!ctx) throw new Error("useLive must be used inside LiveProvider");
  return ctx;
};

// src/events/NotificationBell.tsx
import { useState as useState2 } from "react";
import { Link, useNavigate } from "@tanstack/react-router";
import { Bell, CheckCheck } from "lucide-react";
import { jsx as jsx2, jsxs } from "react/jsx-runtime";
var NotificationBell = ({ topics, feedTo }) => {
  const [open, setOpen] = useState2(false);
  const navigate = useNavigate();
  const { connected } = useLive();
  const { data: countData } = EventsQueries.useUnreadCount();
  const { data: feedData, isLoading } = EventsQueries.useFeed(
    { read: false, limit: 8, sort: "created_at_desc" },
    { enabled: open }
  );
  const { mutate: markRead } = EventsQueries.useAction(EventsApi.markRead);
  const { mutate: markAll, isPending: markingAll } = EventsQueries.useAction(
    EventsApi.markAllRead
  );
  const count = countData?.status === 200 ? countData.data.count : 0;
  const items = feedData?.status === 200 ? feedData.data.items : [];
  const openEvent = (event) => {
    markRead(event.id);
    const link = topics.link(event);
    setOpen(false);
    void navigate(link ? { to: link.to, params: link.params } : { to: feedTo });
  };
  return /* @__PURE__ */ jsxs(Popover, { open, onOpenChange: setOpen, children: [
    /* @__PURE__ */ jsx2(PopoverTrigger, { asChild: true, children: /* @__PURE__ */ jsxs(
      "button",
      {
        type: "button",
        className: "relative rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-ink/5 hover:text-foreground",
        "aria-label": count ? `${count} unread notifications` : "Notifications",
        children: [
          /* @__PURE__ */ jsx2(Bell, { className: "h-4 w-4" }),
          count > 0 && /* @__PURE__ */ jsx2("span", { className: "absolute -right-1 -top-1 flex h-4 min-w-4 items-center justify-center rounded-full bg-danger-600 px-1 text-[0.625rem] font-semibold leading-none text-white tabular-nums", children: count > 99 ? "99+" : count })
        ]
      }
    ) }),
    /* @__PURE__ */ jsxs(
      PopoverContent,
      {
        align: "end",
        sideOffset: 8,
        className: "z-[100] w-96 overflow-hidden rounded-xl border border-ink/15 bg-background-800/95 p-0 shadow-2xl backdrop-blur-xl",
        children: [
          /* @__PURE__ */ jsxs("div", { className: "flex items-center justify-between border-b border-ink/10 px-4 py-3", children: [
            /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-2", children: [
              /* @__PURE__ */ jsx2("span", { className: "text-sm font-semibold", children: "Notifications" }),
              /* @__PURE__ */ jsx2(
                "span",
                {
                  className: cn("h-1.5 w-1.5 rounded-full", connected ? "bg-success-500" : "bg-ink/30"),
                  title: connected ? "Live updates on" : "Live updates reconnecting"
                }
              )
            ] }),
            /* @__PURE__ */ jsxs(
              "button",
              {
                type: "button",
                onClick: () => markAll(void 0),
                disabled: count === 0 || markingAll,
                className: "inline-flex items-center gap-1 text-xs text-brand-sky hover:underline disabled:pointer-events-none disabled:opacity-40",
                children: [
                  /* @__PURE__ */ jsx2(CheckCheck, { className: "h-3.5 w-3.5" }),
                  "Mark all read"
                ]
              }
            )
          ] }),
          /* @__PURE__ */ jsxs("ul", { className: "max-h-96 overflow-y-auto", children: [
            items.length === 0 && /* @__PURE__ */ jsx2("li", { className: "px-4 py-10 text-center text-sm text-ink/50", children: isLoading ? "Loading\u2026" : "You are all caught up." }),
            items.map((event) => /* @__PURE__ */ jsx2("li", { className: "border-b border-ink/[0.06] last:border-0", children: /* @__PURE__ */ jsxs(
              "button",
              {
                type: "button",
                onClick: () => openEvent(event),
                className: "flex w-full items-start gap-3 px-4 py-3 text-left hover:bg-ink/[0.04]",
                children: [
                  /* @__PURE__ */ jsx2("span", { className: "mt-1.5 h-2 w-2 shrink-0 rounded-full bg-brand-sky" }),
                  /* @__PURE__ */ jsxs("span", { className: "min-w-0 flex-1", children: [
                    /* @__PURE__ */ jsx2("span", { className: "block truncate text-sm font-medium", children: topics.describe(event.topic) }),
                    /* @__PURE__ */ jsxs("span", { className: "mt-0.5 flex items-center gap-1.5 text-xs text-ink/50", children: [
                      /* @__PURE__ */ jsx2("span", { className: "truncate font-mono", children: event.topic }),
                      /* @__PURE__ */ jsx2("span", { children: "\xB7" }),
                      /* @__PURE__ */ jsx2(RelativeTime, { date: event.timestamp, className: "shrink-0" })
                    ] })
                  ] })
                ]
              }
            ) }, event.id))
          ] }),
          /* @__PURE__ */ jsx2(
            Link,
            {
              to: feedTo,
              onClick: () => setOpen(false),
              className: "block border-t border-ink/10 px-4 py-2.5 text-center text-xs font-medium text-ink/70 hover:bg-ink/[0.04] hover:text-ink",
              children: "View all events"
            }
          )
        ]
      }
    )
  ] });
};

// src/events/EventFeed.tsx
import { useMemo as useMemo2, useState as useState3 } from "react";
import { Link as Link2 } from "@tanstack/react-router";
import { keepPreviousData } from "@tanstack/react-query";
import { ArrowUpRight, CheckCheck as CheckCheck2 } from "lucide-react";
import { Fragment, jsx as jsx3, jsxs as jsxs2 } from "react/jsx-runtime";
var PERIODS = { hour: 36e5, day: 864e5, week: 6048e5 };
var EventFeed = ({ topics }) => {
  const [selected, setSelected] = useState3(null);
  const { params, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 20,
    defaultSort: "created_at_desc"
  });
  const queryParams = useMemo2(() => {
    const { area, state, period, search: _search, ...rest } = apiParams;
    return {
      ...rest,
      ...area && { topic: area },
      ...state && { read: state === "read" },
      ...period && { created_after: new Date(Date.now() - PERIODS[period]).toISOString() }
    };
  }, [apiParams]);
  const { data, isFetching } = EventsQueries.useFeed(queryParams, {
    placeholderData: keepPreviousData
  });
  const page = data?.status === 200 ? data.data : void 0;
  const { mutate: markRead } = EventsQueries.useAction(EventsApi.markRead);
  const { mutate: markAll, isPending: markingAll } = EventsQueries.useAction(
    EventsApi.markAllRead
  );
  const open = (event) => {
    setSelected(event);
    if (!event.read_at) markRead(event.id);
  };
  return /* @__PURE__ */ jsxs2(Fragment, { children: [
    /* @__PURE__ */ jsx3(
      DataTable,
      {
        data: page,
        serverSide: true,
        loading: isFetching,
        queryParams: params,
        onQueryChange,
        keyExtractor: (e) => e.id,
        onRowClick: open,
        searchable: false,
        emptyMessage: "No events match these filters.",
        toolbarActions: /* @__PURE__ */ jsxs2(Button, { variant: "outline", size: "sm", onClick: () => markAll(void 0), isLoading: markingAll, children: [
          !markingAll && /* @__PURE__ */ jsx3(CheckCheck2, {}),
          "Mark all read"
        ] }),
        filters: [
          {
            id: "area",
            label: "Area",
            options: topics.areas.map((a) => ({ label: a.label, value: a.pattern }))
          },
          {
            id: "state",
            label: "State",
            options: [
              { label: "Unread", value: "unread" },
              { label: "Read", value: "read" }
            ]
          },
          {
            id: "period",
            label: "Period",
            options: [
              { label: "Last hour", value: "hour" },
              { label: "Last 24 hours", value: "day" },
              { label: "Last 7 days", value: "week" }
            ]
          }
        ],
        columns: [
          {
            header: "",
            className: "w-6 pr-0",
            cell: (e) => /* @__PURE__ */ jsx3(
              "span",
              {
                className: cn(
                  "block h-2 w-2 rounded-full",
                  e.read_at ? "bg-transparent" : "bg-brand-sky"
                ),
                title: e.read_at ? "Read" : "Unread"
              }
            )
          },
          {
            header: "Event",
            cell: (e) => /* @__PURE__ */ jsxs2("span", { className: "flex flex-col", children: [
              /* @__PURE__ */ jsx3("span", { className: cn("text-sm", !e.read_at && "font-semibold"), children: topics.describe(e.topic) }),
              /* @__PURE__ */ jsx3("span", { className: "font-mono text-xs text-ink/50", children: e.topic })
            ] })
          },
          {
            header: "Area",
            cell: (e) => /* @__PURE__ */ jsx3(Badge, { variant: "outline", size: "sm", children: topics.area(e.topic) })
          },
          {
            header: "Record",
            cell: (e) => /* @__PURE__ */ jsx3(EventFeed.RecordLink, { event: e, topics })
          },
          {
            header: "When",
            accessorKey: "timestamp",
            sortKey: "created_at",
            cell: (e) => /* @__PURE__ */ jsx3(RelativeTime, { date: e.timestamp, className: "text-sm text-ink/70" })
          }
        ]
      }
    ),
    /* @__PURE__ */ jsx3(Sheet, { open: !!selected, onOpenChange: (o) => !o && setSelected(null), children: /* @__PURE__ */ jsx3(SheetContent, { className: "w-full overflow-y-auto sm:max-w-xl", children: selected && /* @__PURE__ */ jsxs2(Fragment, { children: [
      /* @__PURE__ */ jsxs2(SheetHeader, { className: "mb-4", children: [
        /* @__PURE__ */ jsx3(SheetTitle, { children: topics.describe(selected.topic) }),
        /* @__PURE__ */ jsx3(SheetDescription, { className: "font-mono text-xs", children: selected.topic })
      ] }),
      /* @__PURE__ */ jsx3(
        FieldGrid,
        {
          columns: 2,
          className: "mb-5",
          fields: [
            { label: "Published", value: /* @__PURE__ */ jsx3(RelativeTime, { date: selected.timestamp }) },
            { label: "Source", value: selected.source_crate },
            {
              label: "Record",
              value: /* @__PURE__ */ jsx3(EventFeed.RecordLink, { event: selected, topics })
            },
            { label: "Visibility", value: selected.visibility },
            { label: "Event", value: /* @__PURE__ */ jsx3(Identifier, { value: selected.id }), span: 2 },
            {
              label: "Correlation",
              value: /* @__PURE__ */ jsx3(Identifier, { value: selected.correlation_id }),
              span: 2,
              hidden: !selected.correlation_id
            }
          ]
        }
      ),
      /* @__PURE__ */ jsx3("h3", { className: "mb-2 text-xs font-semibold uppercase tracking-wide text-ink/60", children: "Payload" }),
      /* @__PURE__ */ jsx3(JsonTable, { value: selected.payload })
    ] }) }) })
  ] });
};
EventFeed.RecordLink = ({ event, topics }) => {
  const link = topics.link(event);
  if (!link) return /* @__PURE__ */ jsx3("span", { className: "text-ink/35", children: "\u2014" });
  return /* @__PURE__ */ jsxs2(
    Link2,
    {
      to: link.to,
      params: link.params,
      onClick: (e) => e.stopPropagation(),
      className: "inline-flex items-center gap-1 text-brand-sky hover:underline",
      children: [
        /* @__PURE__ */ jsx3(Identifier, { value: link.label, copyable: false }),
        /* @__PURE__ */ jsx3(ArrowUpRight, { className: "h-3.5 w-3.5" })
      ]
    }
  );
};

// src/events/EventSubscriptions.tsx
import { useState as useState4 } from "react";
import { keepPreviousData as keepPreviousData2 } from "@tanstack/react-query";
import { Plus, Trash2, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { jsx as jsx4, jsxs as jsxs3 } from "react/jsx-runtime";
var CreateSubscriptionDialog = ({
  open,
  onClose,
  topics,
  defaultPattern
}) => {
  const [topicPattern, setTopicPattern] = useState4(defaultPattern);
  const [callbackAddress, setCallbackAddress] = useState4("https://");
  const [secret, setSecret] = useState4("");
  const [retryLimit, setRetryLimit] = useState4("5");
  const topicPresets = [
    ...topics.areas.map((a) => ({ label: a.pattern, pattern: a.pattern })),
    { label: "* (all)", pattern: "*" }
  ];
  const { mutate: createSub, isPending } = EventsQueries.useAction(EventsApi.createSubscription, {
    onSuccess: () => {
      toast.success("Webhook subscription created");
      onClose();
      setSecret("");
    },
    onError: (err) => toast.error(`Subscription creation failed: ${String(err)}`)
  });
  const handleSubmit = () => {
    if (!topicPattern.trim()) {
      toast.error("Topic pattern is required");
      return;
    }
    if (!callbackAddress.trim() || callbackAddress === "https://") {
      toast.error("Valid callback URL is required");
      return;
    }
    const retries = parseInt(retryLimit, 10);
    createSub({
      topic_pattern: topicPattern.trim(),
      callback_address: callbackAddress.trim(),
      secret: secret.trim() || void 0,
      retry_limit: isNaN(retries) ? void 0 : retries
    });
  };
  return /* @__PURE__ */ jsx4(Dialog, { open, onOpenChange: onClose, children: /* @__PURE__ */ jsxs3(DialogContent, { className: "max-w-md", children: [
    /* @__PURE__ */ jsxs3(DialogHeader, { children: [
      /* @__PURE__ */ jsx4(DialogTitle, { children: "Add Webhook Subscription" }),
      /* @__PURE__ */ jsx4(DialogDescription, { className: "text-xs text-muted-foreground", children: "Configure an external webhook listener with automated retry policy and HMAC signing." })
    ] }),
    /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-4 py-2", children: [
      /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-1.5", children: [
        /* @__PURE__ */ jsx4("label", { className: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", children: "Topic Pattern" }),
        /* @__PURE__ */ jsx4(
          Input,
          {
            value: topicPattern,
            onChange: (e) => setTopicPattern(e.target.value),
            placeholder: `e.g. ${defaultPattern}`
          }
        ),
        /* @__PURE__ */ jsxs3("div", { className: "flex flex-wrap items-center gap-1.5 mt-1", children: [
          /* @__PURE__ */ jsx4("span", { className: "text-[11px] text-muted-foreground", children: "Presets:" }),
          topicPresets.map((p) => /* @__PURE__ */ jsx4(
            Button,
            {
              type: "button",
              variant: topicPattern === p.pattern ? "default" : "outline",
              size: "xs",
              className: "text-[11px] h-5 px-2 font-mono",
              onClick: () => setTopicPattern(p.pattern),
              children: p.label
            },
            p.pattern
          ))
        ] }),
        /* @__PURE__ */ jsxs3("p", { className: "text-[11px] text-muted-foreground mt-0.5", children: [
          "Supports ",
          /* @__PURE__ */ jsx4("code", { className: "font-mono", children: ":" }),
          " and ",
          /* @__PURE__ */ jsx4("code", { className: "font-mono", children: "." }),
          " delimiters with ",
          /* @__PURE__ */ jsx4("code", { className: "font-mono", children: "*" }),
          " (single segment) and ",
          /* @__PURE__ */ jsx4("code", { className: "font-mono", children: "**" }),
          " (multi-segment) wildcards."
        ] })
      ] }),
      /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-1.5", children: [
        /* @__PURE__ */ jsx4("label", { className: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", children: "Callback URL" }),
        /* @__PURE__ */ jsx4(
          Input,
          {
            value: callbackAddress,
            onChange: (e) => setCallbackAddress(e.target.value),
            placeholder: "https://your-service.com/webhook"
          }
        )
      ] }),
      /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-1.5", children: [
        /* @__PURE__ */ jsx4("label", { className: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", children: "HMAC-SHA256 Secret (Optional)" }),
        /* @__PURE__ */ jsx4(
          Input,
          {
            type: "password",
            value: secret,
            onChange: (e) => setSecret(e.target.value),
            placeholder: "Leave empty for unauthenticated webhook"
          }
        )
      ] }),
      /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-1.5", children: [
        /* @__PURE__ */ jsx4("label", { className: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", children: "Max Retry Limit" }),
        /* @__PURE__ */ jsx4(
          Input,
          {
            type: "number",
            value: retryLimit,
            onChange: (e) => setRetryLimit(e.target.value),
            min: "1",
            max: "20"
          }
        )
      ] })
    ] }),
    /* @__PURE__ */ jsxs3(DialogFooter, { children: [
      /* @__PURE__ */ jsx4(Button, { variant: "outline", onClick: onClose, disabled: isPending, children: "Cancel" }),
      /* @__PURE__ */ jsx4(Button, { onClick: handleSubmit, disabled: isPending, children: isPending ? "Subscribing..." : "Create Subscription" })
    ] })
  ] }) });
};
var EventSubscriptions = ({ topics, defaultPattern }) => {
  const [createOpen, setCreateOpen] = useState4(false);
  const { params: queryParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { status: "all" }
  });
  const status = queryParams.filters.status;
  const { data, isLoading, isError, isFetching } = EventsQueries.useSubscriptions(
    {
      active: status === "active" ? true : status === "inactive" ? false : void 0,
      limit: queryParams.limit,
      page: queryParams.cursor ? void 0 : queryParams.page,
      cursor: queryParams.cursor,
      sort: queryParams.sort
    },
    { placeholderData: keepPreviousData2 }
  );
  const subscriptions = Array.isArray(data?.data) ? data.data : [];
  const { mutate: deleteSub } = EventsQueries.useAction(EventsApi.deleteSubscription, {
    onSuccess: () => toast.success("Subscription deleted"),
    onError: (err) => toast.error(`Delete failed: ${String(err)}`)
  });
  return /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-6 w-full", children: [
    /* @__PURE__ */ jsx4(
      PageSection,
      {
        title: "Webhooks",
        action: /* @__PURE__ */ jsxs3(Button, { size: "sm", onClick: () => setCreateOpen(true), children: [
          /* @__PURE__ */ jsx4(Plus, {}),
          " New subscription"
        ] }),
        children: isLoading && !data ? /* @__PURE__ */ jsxs3("div", { className: "flex flex-col gap-3 p-4", children: [
          /* @__PURE__ */ jsx4(Skeleton, { className: "h-12 w-full" }),
          /* @__PURE__ */ jsx4(Skeleton, { className: "h-12 w-full" })
        ] }) : isError && !data ? /* @__PURE__ */ jsx4("div", { className: "p-8 text-center text-sm text-destructive", children: "Failed to load event subscriptions." }) : /* @__PURE__ */ jsx4(
          DataTable,
          {
            className: "text-sm",
            data: subscriptions,
            serverSide: true,
            loading: isFetching,
            queryParams,
            onQueryChange,
            keyExtractor: (sub) => sub.id,
            searchPlaceholder: "Filter subscriptions by topic or callback URL...",
            emptyMessage: "No webhook subscriptions configured yet.",
            defaultSortKey: "created_at",
            defaultSortDirection: "desc",
            pageSize: 10,
            pageSizeOptions: [10, 20, 50, 100],
            filters: [
              {
                id: "status",
                label: "Status",
                value: queryParams.filters.status ?? "all",
                options: [
                  { label: "Active", value: "active" },
                  { label: "Inactive", value: "inactive" }
                ]
              }
            ],
            columns: [
              {
                header: "Topic pattern",
                accessorKey: "topic_pattern",
                sortable: false,
                cell: (sub) => /* @__PURE__ */ jsx4(Badge, { variant: "code", children: sub.topic_pattern })
              },
              {
                header: "Status",
                accessorKey: "active",
                sortable: false,
                cell: (sub) => /* @__PURE__ */ jsx4(Badge, { variant: "status", state: sub.active ? "ACTIVE" : "PAUSE", children: sub.active ? "Active" : "Inactive" })
              },
              {
                header: "Callback",
                accessorKey: "callback_address",
                sortable: false,
                cell: (sub) => /* @__PURE__ */ jsx4("span", { className: "font-mono text-xs block max-w-[280px] truncate", children: sub.callback_address })
              },
              {
                header: "Signature",
                sortable: false,
                searchable: false,
                cell: (sub) => sub.secret ? /* @__PURE__ */ jsxs3(Badge, { variant: "success", children: [
                  /* @__PURE__ */ jsx4(ShieldCheck, { className: "h-3 w-3" }),
                  " HMAC"
                ] }) : /* @__PURE__ */ jsx4("span", { className: "text-xs text-muted-foreground", children: "None" })
              },
              {
                header: "Retries",
                accessorKey: "retry_limit",
                sortable: false,
                cell: (sub) => /* @__PURE__ */ jsx4(Badge, { variant: "infoLighter", children: sub.retry_limit ?? 5 })
              },
              {
                header: "Created at",
                accessorKey: "created_at",
                cell: (sub) => /* @__PURE__ */ jsx4(FormatDate, { date: sub.created_at })
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (sub) => /* @__PURE__ */ jsx4(
                  Button,
                  {
                    variant: "ghost",
                    size: "icon",
                    className: "h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10",
                    onClick: () => {
                      if (confirm(`Delete subscription for "${sub.topic_pattern}"?`)) {
                        deleteSub(sub.id);
                      }
                    },
                    children: /* @__PURE__ */ jsx4(Trash2, { className: "h-3.5 w-3.5" })
                  }
                )
              }
            ]
          }
        )
      }
    ),
    /* @__PURE__ */ jsx4(
      CreateSubscriptionDialog,
      {
        open: createOpen,
        onClose: () => setCreateOpen(false),
        topics,
        defaultPattern
      }
    )
  ] });
};

// src/events/DeadLetters.tsx
import { useState as useState5 } from "react";
import { keepPreviousData as keepPreviousData3 } from "@tanstack/react-query";
import { AlertTriangle, RefreshCw, Trash2 as Trash22, Eye, RotateCcw } from "lucide-react";
import { toast as toast2 } from "sonner";
import { jsx as jsx5, jsxs as jsxs4 } from "react/jsx-runtime";
var DeadLetters = () => {
  const [selectedDlq, setSelectedDlq] = useState5(null);
  const { params: queryParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { status: "all" }
  });
  const selectedStatus = queryParams.filters.status && queryParams.filters.status !== "all" ? queryParams.filters.status : void 0;
  const { data, isLoading, isError, refetch, isFetching } = EventsQueries.useDeadLetters(
    {
      status: selectedStatus,
      limit: queryParams.limit,
      page: queryParams.cursor ? void 0 : queryParams.page,
      cursor: queryParams.cursor,
      sort: queryParams.sort
    },
    { placeholderData: keepPreviousData3 }
  );
  const deadLetters = Array.isArray(data?.data) ? data.data : [];
  const { mutate: replaySingle, isPending: isReplayingSingle } = EventsQueries.useAction(
    EventsApi.replayDeadLetter,
    {
      onSuccess: () => toast2.success("Event re-enqueued for delivery"),
      onError: (err) => toast2.error(`Replay failed: ${String(err)}`)
    }
  );
  const { mutate: replayAll, isPending: isReplayingAll } = EventsQueries.useAction(
    EventsApi.replayAllDeadLetters,
    {
      onSuccess: (res) => toast2.success(`Replayed ${res.data.replayed_count} dead letters`),
      onError: (err) => toast2.error(`Replay all failed: ${String(err)}`)
    }
  );
  const { mutate: deleteRecord } = EventsQueries.useAction(EventsApi.deleteDeadLetter, {
    onSuccess: () => toast2.success("Dead letter record discarded"),
    onError: (err) => toast2.error(`Discard failed: ${String(err)}`)
  });
  return /* @__PURE__ */ jsxs4("div", { className: "flex flex-col gap-6 w-full", children: [
    /* @__PURE__ */ jsx5("div", { className: "flex flex-col sm:flex-row sm:items-center justify-end gap-4", children: /* @__PURE__ */ jsxs4("div", { className: "flex items-center gap-2", children: [
      /* @__PURE__ */ jsxs4(
        Button,
        {
          variant: "outline",
          size: "sm",
          onClick: () => refetch(),
          disabled: isFetching,
          className: "gap-1.5",
          children: [
            /* @__PURE__ */ jsx5(RefreshCw, { className: `h-3.5 w-3.5 ${isFetching ? "animate-spin" : ""}` }),
            "Refresh"
          ]
        }
      ),
      /* @__PURE__ */ jsxs4(
        Button,
        {
          size: "sm",
          onClick: () => replayAll(void 0),
          disabled: isReplayingAll || deadLetters.length === 0,
          className: "gap-1.5",
          children: [
            /* @__PURE__ */ jsx5(RotateCcw, { className: "h-3.5 w-3.5" }),
            isReplayingAll ? "Replaying All..." : "Replay All"
          ]
        }
      )
    ] }) }),
    /* @__PURE__ */ jsx5("div", { className: "flex items-center gap-2", children: [
      { label: "Unresolved", value: "Unresolved" },
      { label: "Replayed", value: "Replayed" },
      { label: "Purged", value: "Purged" }
    ].map((item) => /* @__PURE__ */ jsx5(
      Button,
      {
        variant: (queryParams.filters.status ?? "all") === item.value ? "default" : "outline",
        size: "sm",
        className: "text-xs h-7",
        onClick: () => onQueryChange({ filters: { status: item.value }, page: 1 }),
        children: item.label
      },
      item.value
    )) }),
    /* @__PURE__ */ jsx5(PageSection, { children: isLoading && !data ? /* @__PURE__ */ jsxs4("div", { className: "flex flex-col gap-3 p-4", children: [
      /* @__PURE__ */ jsx5(Skeleton, { className: "h-12 w-full" }),
      /* @__PURE__ */ jsx5(Skeleton, { className: "h-12 w-full" })
    ] }) : isError && !data ? /* @__PURE__ */ jsx5("div", { className: "p-8 text-center text-sm text-destructive", children: "Failed to load Dead Letter Queue records." }) : /* @__PURE__ */ jsx5(
      DataTable,
      {
        className: "text-sm",
        data: deadLetters,
        serverSide: true,
        loading: isFetching,
        queryParams,
        onQueryChange,
        keyExtractor: (dlq) => dlq.id,
        searchPlaceholder: "Filter dead letters by topic, status, or callback...",
        emptyMessage: "No dead letter messages found. All deliveries healthy.",
        defaultSortKey: "created_at",
        defaultSortDirection: "desc",
        pageSize: 10,
        pageSizeOptions: [10, 20, 50, 100],
        filters: [
          {
            id: "status",
            label: "Status",
            value: queryParams.filters.status ?? "all",
            options: [
              { label: "Unresolved", value: "Unresolved" },
              { label: "Replayed", value: "Replayed" },
              { label: "Purged", value: "Purged" }
            ]
          }
        ],
        columns: [
          {
            header: "Topic",
            accessorKey: "topic",
            sortable: false,
            cell: (dlq) => /* @__PURE__ */ jsx5(Badge, { variant: "code", children: dlq.topic })
          },
          {
            header: "Status",
            accessorKey: "status",
            sortable: false,
            cell: (dlq) => /* @__PURE__ */ jsx5(Badge, { variant: dlq.status === "Unresolved" ? "destructive" : "default", children: dlq.status })
          },
          {
            header: "Error",
            accessorKey: "error_message",
            sortable: false,
            cell: (dlq) => /* @__PURE__ */ jsx5("span", { className: "text-danger-700 dark:text-danger-300 text-xs block max-w-[280px] truncate", children: dlq.error_message })
          },
          {
            header: "Callback",
            accessorKey: "callback_address",
            sortable: false,
            cell: (dlq) => /* @__PURE__ */ jsx5("span", { className: "font-mono text-xs text-muted-foreground block max-w-[240px] truncate", children: dlq.callback_address })
          },
          {
            header: "Attempts",
            accessorKey: "attempts",
            sortable: false,
            cell: (dlq) => /* @__PURE__ */ jsx5(Badge, { variant: "infoLighter", children: dlq.attempts })
          },
          {
            header: "Failed at",
            sortKey: "created_at",
            cell: (dlq) => /* @__PURE__ */ jsx5(FormatDate, { date: dlq.failed_at })
          },
          {
            header: "Actions",
            sortable: false,
            searchable: false,
            cell: (dlq) => /* @__PURE__ */ jsxs4("div", { className: "flex items-center gap-2", children: [
              /* @__PURE__ */ jsxs4(
                Button,
                {
                  variant: "outline",
                  size: "xs",
                  className: "gap-1",
                  disabled: isReplayingSingle,
                  onClick: () => replaySingle(dlq.id),
                  children: [
                    /* @__PURE__ */ jsx5(RotateCcw, { className: "h-3 w-3" }),
                    " Replay"
                  ]
                }
              ),
              /* @__PURE__ */ jsx5(
                Button,
                {
                  variant: "ghost",
                  size: "icon",
                  className: "h-7 w-7",
                  onClick: () => setSelectedDlq(dlq),
                  children: /* @__PURE__ */ jsx5(Eye, { className: "h-3.5 w-3.5" })
                }
              ),
              /* @__PURE__ */ jsx5(
                Button,
                {
                  variant: "ghost",
                  size: "icon",
                  className: "h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10",
                  onClick: () => {
                    if (confirm("Discard this dead letter record?")) {
                      deleteRecord(dlq.id);
                    }
                  },
                  children: /* @__PURE__ */ jsx5(Trash22, { className: "h-3.5 w-3.5" })
                }
              )
            ] })
          }
        ]
      }
    ) }),
    /* @__PURE__ */ jsx5(Dialog, { open: !!selectedDlq, onOpenChange: () => setSelectedDlq(null), children: /* @__PURE__ */ jsxs4(DialogContent, { className: "max-w-2xl max-h-[85vh] flex flex-col", children: [
      /* @__PURE__ */ jsxs4(DialogHeader, { children: [
        /* @__PURE__ */ jsxs4(DialogTitle, { className: "flex items-center gap-2 text-sm font-mono text-destructive", children: [
          /* @__PURE__ */ jsx5(AlertTriangle, { className: "h-4 w-4" }),
          "Dead Letter: ",
          selectedDlq?.id
        ] }),
        /* @__PURE__ */ jsx5(DialogDescription, { className: "text-xs text-muted-foreground", children: "Dead letter queue inspection for failed event delivery." })
      ] }),
      /* @__PURE__ */ jsxs4("div", { className: "flex-1 overflow-y-auto rounded bg-muted/40 p-4 font-mono text-xs flex flex-col gap-3", children: [
        /* @__PURE__ */ jsxs4("div", { children: [
          /* @__PURE__ */ jsx5("span", { className: "text-muted-foreground font-semibold", children: "Error Message:" }),
          /* @__PURE__ */ jsx5("p", { className: "text-destructive mt-1 font-sans", children: selectedDlq?.error_message })
        ] }),
        /* @__PURE__ */ jsxs4("div", { children: [
          /* @__PURE__ */ jsx5("span", { className: "text-muted-foreground font-semibold", children: "Target Callback:" }),
          /* @__PURE__ */ jsx5("p", { className: "text-foreground mt-1 select-all", children: selectedDlq?.callback_address })
        ] }),
        /* @__PURE__ */ jsxs4("div", { children: [
          /* @__PURE__ */ jsx5("span", { className: "text-muted-foreground font-semibold", children: "Event Payload:" }),
          /* @__PURE__ */ jsx5("pre", { className: "whitespace-pre-wrap break-all mt-1 bg-background/60 p-2.5 rounded border border-ink/5", children: JSON.stringify(selectedDlq?.payload, null, 2) })
        ] })
      ] }),
      /* @__PURE__ */ jsx5(DialogFooter, { children: /* @__PURE__ */ jsx5(Button, { onClick: () => setSelectedDlq(null), children: "Close" }) })
    ] }) })
  ] });
};

// src/events/EventsLayout.tsx
import { jsx as jsx6, jsxs as jsxs5 } from "react/jsx-runtime";
var EventsLayout = ({ basePath, description, children }) => /* @__PURE__ */ jsxs5(PageLayout, { children: [
  /* @__PURE__ */ jsx6(PageHeader, { title: "Events", description }),
  /* @__PURE__ */ jsx6(
    RouteTabs,
    {
      tabs: [
        { label: "Feed", to: `${basePath}/feed` },
        { label: "Webhook subscriptions", to: `${basePath}/subscriptions` },
        { label: "Dead letters", to: `${basePath}/dlq` }
      ]
    }
  ),
  children
] });
export {
  DeadLetters,
  EventFeed,
  EventSubscriptions,
  EventsApi,
  EventsLayout,
  EventsQueries,
  EventsQueryKeys,
  LiveProvider,
  NotificationBell,
  useLive
};
//# sourceMappingURL=index.js.map