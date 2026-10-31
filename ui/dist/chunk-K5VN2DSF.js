import {
  cn
} from "./chunk-4GNLXNMN.js";

// src/link.tsx
import React, { createContext, useContext } from "react";
import { Link as TanStackLink, useRouterState } from "@tanstack/react-router";
import { jsx } from "react/jsx-runtime";
var DefaultTanStackLink = React.forwardRef(
  ({ to, ...rest }, ref) => /* @__PURE__ */ jsx(TanStackLink, { ref, to, ...rest })
);
DefaultTanStackLink.displayName = "DefaultTanStackLink";
var LinkContext = createContext(DefaultTanStackLink);
var ActivePathContext = createContext(null);
var LinkProvider = LinkContext.Provider;
var ActivePathProvider = ActivePathContext.Provider;
var useLink = () => useContext(LinkContext);
var useActivePath = () => {
  const ctxPath = useContext(ActivePathContext);
  const routerPath = useRouterState({ select: (s) => s.location.pathname });
  return ctxPath ?? routerPath ?? "";
};
function NavigationProvider({
  link = DefaultTanStackLink,
  activePath,
  children
}) {
  return /* @__PURE__ */ jsx(LinkContext.Provider, { value: link, children: /* @__PURE__ */ jsx(ActivePathContext.Provider, { value: activePath ?? null, children }) });
}

// src/hooks/useTableQueryParams.ts
import { useState, useMemo, useCallback, useEffect } from "react";
function mapToSortParam(key, direction) {
  if (key === "createdAt" || key === "created_at") {
    return direction === "asc" ? "created_at_asc" : "created_at_desc";
  }
  if (key === "updatedAt" || key === "updated_at" || key === "last_interaction") {
    return direction === "asc" ? "updated_at_asc" : "updated_at_desc";
  }
  const snakeKey = key.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
  return `${snakeKey}_${direction}`;
}
function useTableQueryParams(options = {}) {
  const {
    defaultLimit = 10,
    defaultSort = "created_at_desc",
    defaultFilters = {},
    syncUrl = true
  } = options;
  const [params, setParams] = useState(() => {
    if (typeof window === "undefined") {
      return {
        limit: defaultLimit,
        page: 1,
        cursor: void 0,
        sort: defaultSort,
        search: "",
        filters: { ...defaultFilters }
      };
    }
    const sp = new URLSearchParams(window.location.search);
    const limit = Number(sp.get("limit")) || defaultLimit;
    const page = Number(sp.get("page")) || 1;
    const cursor = sp.get("cursor") || void 0;
    const sort = sp.get("sort") || defaultSort;
    const search = sp.get("search") || "";
    const filters = { ...defaultFilters };
    sp.forEach((value, key) => {
      if (!["limit", "page", "cursor", "sort", "search"].includes(key)) {
        filters[key] = value;
      }
    });
    return { limit, page, cursor, sort, search, filters };
  });
  useEffect(() => {
    if (!syncUrl || typeof window === "undefined") return;
    const url = new URL(window.location.href);
    const sp = url.searchParams;
    if (params.limit !== defaultLimit) {
      sp.set("limit", String(params.limit));
    } else {
      sp.delete("limit");
    }
    if (params.page > 1) {
      sp.set("page", String(params.page));
    } else {
      sp.delete("page");
    }
    if (params.cursor) {
      sp.set("cursor", params.cursor);
    } else {
      sp.delete("cursor");
    }
    if (params.sort && params.sort !== defaultSort) {
      sp.set("sort", params.sort);
    } else {
      sp.delete("sort");
    }
    if (params.search && params.search.trim()) {
      sp.set("search", params.search.trim());
    } else {
      sp.delete("search");
    }
    Object.entries(params.filters).forEach(([key, value]) => {
      if (value && value !== "all") {
        sp.set(key, value);
      } else {
        sp.delete(key);
      }
    });
    const newSearch = sp.toString();
    const newRelativePath = url.pathname + (newSearch ? `?${newSearch}` : "");
    window.history.replaceState(null, "", newRelativePath);
  }, [params, defaultLimit, defaultSort, syncUrl]);
  const apiParams = useMemo(() => {
    const p = {
      limit: params.limit,
      sort: params.sort
    };
    if (params.cursor) {
      p.cursor = params.cursor;
    } else if (params.page > 1) {
      p.page = params.page;
    }
    if (params.search && params.search.trim()) {
      p.search = params.search.trim();
    }
    Object.entries(params.filters).forEach(([k, v]) => {
      if (v && v !== "all") {
        p[k] = v;
      }
    });
    return p;
  }, [params]);
  const onQueryChange = useCallback((updated) => {
    setParams((prev) => {
      const nextFilters = updated.filters ? { ...prev.filters, ...updated.filters } : prev.filters;
      return {
        ...prev,
        ...updated,
        filters: nextFilters
      };
    });
  }, []);
  const setPage = useCallback((page) => {
    setParams((p) => ({ ...p, page, cursor: void 0 }));
  }, []);
  const setLimit = useCallback((limit) => {
    setParams((p) => ({ ...p, limit, page: 1, cursor: void 0 }));
  }, []);
  const setSort = useCallback((sort) => {
    setParams((p) => ({ ...p, sort, page: 1, cursor: void 0 }));
  }, []);
  const setSearch = useCallback((search) => {
    setParams((p) => ({ ...p, search, page: 1, cursor: void 0 }));
  }, []);
  const setFilter = useCallback((id, value) => {
    setParams((p) => ({
      ...p,
      page: 1,
      cursor: void 0,
      filters: { ...p.filters, [id]: value }
    }));
  }, []);
  const resetAll = useCallback(() => {
    setParams({
      limit: defaultLimit,
      page: 1,
      cursor: void 0,
      sort: defaultSort,
      search: "",
      filters: { ...defaultFilters }
    });
  }, [defaultLimit, defaultSort, defaultFilters]);
  return {
    params,
    apiParams,
    onQueryChange,
    setPage,
    setLimit,
    setSort,
    setSearch,
    setFilter,
    resetAll
  };
}

// src/lib/http.ts
var UNAUTHORIZED_EVENT = "eunomia:unauthorized";
var API_GATEWAY_BASE = "";
var setApiGatewayBase = (url) => {
  API_GATEWAY_BASE = url;
};
var getApiGatewayBase = () => API_GATEWAY_BASE;
var customInstance = async (url, options) => {
  const { method, headers, params, data, _noSessionRedirect, ...rest } = options || {};
  const normalizedHeaders = {
    "Content-Type": "application/json"
  };
  if (headers) {
    if (headers instanceof Headers) {
      headers.forEach((v, k) => {
        normalizedHeaders[k] = v;
      });
    } else if (Array.isArray(headers)) {
      headers.forEach(([k, v]) => {
        normalizedHeaders[k] = v;
      });
    } else {
      Object.entries(headers).forEach(([k, v]) => {
        if (v !== void 0) {
          normalizedHeaders[k] = Array.isArray(v) ? v.join(", ") : String(v);
        }
      });
    }
  }
  const config = {
    ...rest,
    headers: normalizedHeaders
  };
  let targetUrl = url;
  if (params) {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== void 0 && value !== null) {
        if (Array.isArray(value)) {
          value.forEach((v) => searchParams.append(key, String(v)));
        } else {
          searchParams.append(key, String(value));
        }
      }
    });
    const queryString = searchParams.toString();
    if (queryString) {
      targetUrl += (targetUrl.includes("?") ? "&" : "?") + queryString;
    }
  }
  const fullUrl = API_GATEWAY_BASE ? `${API_GATEWAY_BASE}${targetUrl}` : targetUrl;
  const requestMethod = method || "GET";
  const fetchOptions = {
    ...config,
    method: requestMethod
  };
  if (data !== void 0) {
    fetchOptions.body = JSON.stringify(data);
  }
  const response = await fetch(fullUrl, fetchOptions);
  let parsedData;
  if (response.status === 204) {
    parsedData = {};
  } else {
    try {
      parsedData = await response.clone().json();
    } catch {
      parsedData = await response.text();
    }
  }
  if (response.status === 401 && !_noSessionRedirect && typeof window !== "undefined") {
    window.dispatchEvent(new Event(UNAUTHORIZED_EVENT));
  }
  if (parsedData && typeof parsedData === "object" && !Array.isArray(parsedData) && Array.isArray(parsedData.items)) {
    const envelope = parsedData;
    const arr = [...envelope.items];
    arr.items = envelope.items;
    arr.total = envelope.total;
    arr.nextCursor = envelope.nextCursor;
    parsedData = arr;
  }
  return {
    status: response.status,
    data: parsedData,
    headers: response.headers
  };
};

// src/ui/badge.tsx
import { Slot } from "@radix-ui/react-slot";
import { cva } from "class-variance-authority";
import { jsx as jsx2, jsxs } from "react/jsx-runtime";
var normalizeStatus = (status) => {
  switch ((status || "").toLowerCase()) {
    case "active":
    case "accepted":
    case "verified":
    case "started":
    case "approved":
    case "agreed":
    case "in_progress":
    case "running":
    case "valid":
      return "process";
    case "offered":
    case "requested":
    case "pending":
    case "processing":
    case "waiting":
    case "expiring":
    case "warning":
      return "warn";
    case "finalized":
    case "completed":
    case "success":
    case "confirmed":
    case "delivered":
    case "delivered_confirmed":
    case "ok":
      return "success";
    case "inactive":
    case "suspended":
    case "pause":
    case "paused":
    case "by_provider":
    case "by_consumer":
    case "on_request":
    case "stop":
    case "stopped":
    case "idle":
      return "pause";
    case "terminated":
    case "rejected":
    case "failed":
    case "error":
    case "revoked":
    case "dead_letter":
    case "dlq":
    case "expired":
      return "danger";
    default:
      return "default";
  }
};
var badgeVariants = cva(
  "px-2 py-0.5 w-fit inline-flex justify-start items-center font-medium rounded-md border whitespace-nowrap shrink-0 gap-1.5 transition-all text-xs tracking-tight",
  {
    variants: {
      variant: {
        default: "bg-ink/10 text-brand-snow border-ink/15 hover:bg-ink/15",
        secondary: "bg-secondary-100 dark:bg-secondary-800/40 text-secondary-700 dark:text-secondary-200 border-secondary-600/40",
        outline: "bg-transparent text-foreground-200 border-ink/20 hover:border-ink/30",
        destructive: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30",
        danger: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30",
        success: "bg-success-500/15 text-success-700 dark:text-success-300 border-success-500/30",
        warning: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30",
        warn: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30",
        info: "font-mono bg-primary-100 dark:bg-primary-900/40 text-brand-sky border-primary-600/30",
        infoLighter: "font-mono bg-ink/10 text-secondary-700 dark:text-secondary-300 border-ink/15",
        role: "text-ink uppercase border-ink/15",
        status: "border-ink/15 text-foreground-300 uppercase tracking-wide",
        detail: "text-xs bg-brand-sky/15 text-brand-sky border-brand-sky/30 !px-1.5 !py-0.5 max-w-[140px] !whitespace-normal",
        code: "bg-sunken/60 border border-danger-200 dark:border-danger-900/40 rounded font-mono text-danger-700 dark:text-danger-400 !py-0 px-1.5",
        wizard: "bg-violet-100 dark:bg-violet-900/50 border border-violet-600/60 text-violet-700 dark:text-violet-200 uppercase tracking-wide px-3 py-0.5",
        wizardSuccess: "bg-success-100 dark:bg-success-900/50 border border-success-600/60 text-success-800 dark:text-success-200 uppercase tracking-wide px-3 py-0.5"
      },
      state: {
        default: "",
        process: "bg-process-400/15 text-process-800 dark:text-process-300 border-process-400/30 [&>span]:bg-process-400",
        warn: "bg-warn-500/15 text-warn-700 dark:text-warn-300 border-warn-500/30 [&>span]:bg-warn-400",
        success: "bg-success-500/15 text-success-700 dark:text-success-300 border-success-500/30 [&>span]:bg-success-400",
        pause: "bg-pause-500/15 text-pause-700 dark:text-pause-300 border-pause-500/30 [&>span]:bg-pause-400",
        danger: "bg-danger-500/15 text-danger-700 dark:text-danger-300 border-danger-500/30 [&>span]:bg-danger-400"
      },
      dsrole: {
        Provider: "bg-roles-provider/15 text-roles-provider border-roles-provider/30",
        Consumer: "bg-roles-consumer/15 text-roles-consumer border-roles-consumer/30",
        Business: "bg-roles-bussiness/15 text-roles-bussiness border-roles-bussiness/30",
        Customer: "bg-roles-customer/15 text-roles-customer border-roles-customer/30"
      },
      size: {
        xs: "text-xs px-1.5 py-0 leading-tight font-medium rounded",
        sm: "text-xs px-2 py-0.5 leading-tight font-medium rounded",
        default: "text-xs px-2.5 py-0.5 font-medium rounded-md",
        lg: "text-sm px-3 py-1 font-semibold rounded-md"
      }
    },
    defaultVariants: {
      variant: "default",
      state: "default",
      size: "default"
    }
  }
);
function Badge({
  className,
  variant,
  state,
  size,
  dsrole,
  dot,
  asChild = false,
  children,
  ...props
}) {
  const Comp = asChild ? Slot : "span";
  let resolvedRole = dsrole;
  if (!resolvedRole && variant === "role" && typeof children === "string") {
    const roleStr = children.trim().toLowerCase();
    if (roleStr === "provider") resolvedRole = "Provider";
    else if (roleStr === "consumer") resolvedRole = "Consumer";
    else if (roleStr === "business") resolvedRole = "Business";
    else if (roleStr === "customer") resolvedRole = "Customer";
  }
  const showDot = variant === "status" || dot === true;
  const stateStyle = normalizeStatus(state);
  return /* @__PURE__ */ jsxs(
    Comp,
    {
      "data-slot": "badge",
      className: cn(
        badgeVariants({ variant, size, state: stateStyle, dsrole: resolvedRole }),
        className
      ),
      ...props,
      children: [
        showDot && /* @__PURE__ */ jsx2("span", { className: "w-1.5 h-1.5 rounded-full mr-1 shrink-0 inline-block bg-current shadow-[0_0_6px_currentColor]" }),
        children
      ]
    }
  );
}

// src/ui/button.tsx
import * as React2 from "react";
import { Loader2 } from "lucide-react";
import { Slot as Slot2 } from "@radix-ui/react-slot";
import { cva as cva2 } from "class-variance-authority";
import { Fragment, jsx as jsx3, jsxs as jsxs2 } from "react/jsx-runtime";
var buttonVariants = cva2(
  "inline-flex items-center flex-nowrap min-w-fit !leading-none justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-sky/40 focus-visible:ring-offset-1 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary-700 text-white hover:bg-primary-600 border border-primary-500/30 hover:border-primary-400/50 shadow-sm active:scale-[0.98]",
        destructive: "border border-danger-700/50 bg-danger-50 dark:bg-danger-950/40 text-danger-700 dark:text-danger-300 hover:bg-danger-100 dark:hover:bg-danger-900/60 hover:text-danger-800 dark:hover:text-danger-100 hover:border-danger-600/70 shadow-sm active:scale-[0.98]",
        outline: "border border-ink/20 text-ink/90 bg-transparent hover:bg-ink/10 hover:text-ink hover:border-ink/30 shadow-sm active:scale-[0.98]",
        outline_destructive: "border border-danger-700/60 text-danger-700 dark:text-danger-300 bg-transparent hover:bg-danger-50 dark:bg-danger-950/40 hover:text-danger-800 dark:hover:text-danger-200 hover:border-danger-600 active:scale-[0.98]",
        secondary: "bg-ink/10 text-ink hover:bg-ink/15 border border-ink/15 hover:border-ink/25 shadow-sm active:scale-[0.98]",
        ghost: "text-ink/80 hover:text-ink hover:bg-ink/10 active:scale-[0.98]",
        success: "border border-success-700/50 bg-success-50 dark:bg-success-950/40 text-success-700 dark:text-success-300 hover:bg-success-100 dark:hover:bg-success-900/60 hover:text-success-800 dark:hover:text-success-100 hover:border-success-600/70 shadow-sm active:scale-[0.98]",
        icon_destructive: "flex text-danger-700 dark:text-danger-400 border border-danger-700/40 bg-danger-50 dark:bg-danger-950/20 hover:bg-danger-100 dark:hover:bg-danger-900/50 hover:text-danger-800 dark:hover:text-danger-200 active:scale-[0.98] p-1.5",
        link: "!px-0 flex-nowrap normal-case text-brand-sky underline-offset-4 hover:underline hover:text-ink active:scale-100 transition-colors"
      },
      size: {
        default: "h-9 px-4",
        sm: "h-8 px-3 text-[0.8125rem]",
        xs: "h-7 px-2.5 text-xs [&_svg]:size-3.5",
        lg: "h-10 px-6",
        icon: "h-9 w-9 p-0",
        icon_sm: "h-8 w-8 p-0 [&_svg]:size-3.5"
      },
      policy: {
        default: "",
        permission: "bg-success-600/20 border-success-700 text-success-800 dark:text-success-100 hover:bg-success-600/30",
        obligation: "bg-warn-700/20 border-warn-700 text-warn-800 dark:text-warn-100 hover:bg-warn-700/30",
        prohibition: "bg-danger-600/20 border-danger-700 text-danger-800 dark:text-danger-100 hover:bg-danger-600/30"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "default",
      policy: "default"
    }
  }
);
var Button = React2.forwardRef(
  ({ className, variant, size, policy, asChild = false, isLoading, ...props }, ref) => {
    const Comp = asChild ? Slot2 : "button";
    return /* @__PURE__ */ jsx3(
      Comp,
      {
        className: cn(buttonVariants({ variant, size, policy, className })),
        ref,
        disabled: isLoading || props.disabled,
        ...props,
        children: asChild ? props.children : /* @__PURE__ */ jsxs2(Fragment, { children: [
          isLoading && /* @__PURE__ */ jsx3(Loader2, { className: "animate-spin" }),
          props.children
        ] })
      }
    );
  }
);
Button.displayName = "Button";

// src/ui/dialog.tsx
import * as React3 from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { Cross2Icon } from "@radix-ui/react-icons";
import { jsx as jsx4, jsxs as jsxs3 } from "react/jsx-runtime";
var Dialog = DialogPrimitive.Root;
var DialogTrigger = DialogPrimitive.Trigger;
var DialogPortal = DialogPrimitive.Portal;
var DialogClose = DialogPrimitive.Close;
var DialogOverlay = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx4(
  DialogPrimitive.Overlay,
  {
    ref,
    className: cn(
      "fixed inset-0 z-50 bg-black/80 flex data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className
    ),
    ...props
  }
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;
var DialogContent = React3.forwardRef(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs3(DialogPortal, { children: [
  /* @__PURE__ */ jsx4(DialogOverlay, {}),
  /* @__PURE__ */ jsxs3(
    DialogPrimitive.Content,
    {
      ref,
      className: cn(
        "sm:min-w-[600px] fixed left-[50%] top-[50%] z-50 w-full grid overflow-hidden max-w-lg translate-x-[-50%] translate-y-[-50%] gap-6 border bg-background p-6 text-text shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",
        className
      ),
      ...props,
      children: [
        children,
        /* @__PURE__ */ jsxs3(DialogPrimitive.Close, { className: "absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground", children: [
          /* @__PURE__ */ jsx4(Cross2Icon, { className: "h-4 w-4" }),
          /* @__PURE__ */ jsx4("span", { className: "sr-only", children: "Close" })
        ] })
      ]
    }
  )
] }));
DialogContent.displayName = DialogPrimitive.Content.displayName;
var DialogHeader = ({ className, ...props }) => /* @__PURE__ */ jsx4("div", { className: cn("flex flex-col space-y-1.5 sm:text-left", className), ...props });
DialogHeader.displayName = "DialogHeader";
var DialogBody = ({ className, ...props }) => /* @__PURE__ */ jsx4("div", { className: cn("py-4 overflow-y-scroll px-6", className), ...props });
DialogBody.displayName = "DialogBody";
var DialogFooter = ({ className, ...props }) => /* @__PURE__ */ jsx4(
  "div",
  {
    className: cn("flex flex-col-reverse gap-2 xs:flex-row xs:justify-end xs:space-x-2", className),
    ...props
  }
);
DialogFooter.displayName = "DialogFooter";
var DialogTitle = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx4(
  DialogPrimitive.Title,
  {
    ref,
    className: cn(
      "mb-2 text-lg text-foreground-400 font-medium leading-none tracking-tight",
      className
    ),
    ...props
  }
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;
var DialogDescription = React3.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx4(
  DialogPrimitive.Description,
  {
    ref,
    className: cn("text-base text-muted-foreground", className),
    ...props
  }
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

// src/ui/count-badge.tsx
import { jsx as jsx5 } from "react/jsx-runtime";
function CountBadge({
  n,
  showZero = false,
  className
}) {
  if (n === void 0 || n === 0 && !showZero) return null;
  return /* @__PURE__ */ jsx5(
    "span",
    {
      className: cn("rounded-full bg-ink/10 px-1.5 text-xs font-medium tabular-nums", className),
      children: n
    }
  );
}

// src/ui/format-date.tsx
import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import { jsx as jsx6 } from "react/jsx-runtime";
dayjs.extend(relativeTime);
var DATE_TIME_FORMAT = "YYYY-MM-DD HH:mm";
var DATE_FORMAT = "YYYY-MM-DD";
var FormatDate = ({ date, format = DATE_TIME_FORMAT }) => {
  if (!date) return /* @__PURE__ */ jsx6("span", { className: "text-ink/35", children: "\u2014" });
  return /* @__PURE__ */ jsx6("time", { dateTime: dayjs(date).toISOString(), title: dayjs(date).format("YYYY-MM-DD HH:mm:ss"), children: dayjs(date).format(format) });
};
var RelativeTime = ({ date, className }) => /* @__PURE__ */ jsx6(
  "time",
  {
    dateTime: dayjs(date).toISOString(),
    title: dayjs(date).format("YYYY-MM-DD HH:mm:ss"),
    className,
    children: dayjs(date).fromNow()
  }
);

// src/ui/heading.tsx
import { jsx as jsx7 } from "react/jsx-runtime";
var levelToElement = {
  h1: "h1",
  h2: "h2",
  h3: "h3",
  h4: "h4",
  h5: "h5",
  h6: "h6",
  table: "h6",
  "title-sm": "h6",
  subtitle: "h5"
};
var sizeClasses = {
  h1: "text-2xl mb-4 font-semibold font-title tracking-tight",
  h2: "text-xl mb-3 font-semibold tracking-tight",
  h3: "text-lg mb-3 font-medium tracking-tight",
  h4: "text-base mb-2 font-medium font-display tracking-normal text-ink/90",
  h5: "text-sm text-ink/80 mb-2 font-medium tracking-wide",
  h6: "text-xs font-semibold mb-1 uppercase tracking-wider text-muted-foreground",
  table: "text-xs font-semibold uppercase tracking-wider",
  "title-sm": "text-sm font-medium mb-1 leading-snug",
  subtitle: "text-base mb-2 font-normal text-muted-foreground max-w-[65ch]"
};
var Heading = ({ level = "h1", children, ref, className = "" }) => {
  const Component = levelToElement[level] || "h1";
  const baseClasses = "text-foreground-100 text-balance";
  return /* @__PURE__ */ jsx7(Component, { className: cn(baseClasses, sizeClasses[level], className), children });
};
var heading_default = Heading;

// src/ui/input.tsx
import * as React4 from "react";
import { Search, X } from "lucide-react";
import { jsx as jsx8, jsxs as jsxs4 } from "react/jsx-runtime";
var Input = React4.forwardRef(
  ({
    className,
    containerClassName,
    type = "text",
    placeholder,
    startIcon,
    endIcon,
    onClear,
    value,
    ...props
  }, ref) => {
    const isSearch = type === "search";
    const hasDecorator = Boolean(startIcon || endIcon || onClear || isSearch);
    const baseInputStyles = "w-full rounded-md border border-ink/15 bg-background-800/90 px-3 py-1.5 text-xs text-brand-snow placeholder:text-muted-foreground/60 transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-brand-sky/60 focus-visible:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]";
    if (!hasDecorator) {
      return /* @__PURE__ */ jsx8(
        "input",
        {
          ...props,
          ref,
          type,
          value,
          placeholder,
          className: cn("flex h-9", baseInputStyles, className)
        }
      );
    }
    const defaultPlaceholder = isSearch ? "Search..." : void 0;
    const resolvedPlaceholder = placeholder !== void 0 ? placeholder : defaultPlaceholder;
    return /* @__PURE__ */ jsxs4(
      "div",
      {
        className: cn(
          "flex h-9 w-full items-center rounded-md border border-ink/15 bg-background-800/90 px-2.5 text-xs text-brand-snow transition-all focus-within:border-brand-sky/60 focus-within:ring-1 focus-within:ring-brand-sky/60 hover:border-ink/25",
          containerClassName
        ),
        children: [
          startIcon ? /* @__PURE__ */ jsx8("span", { className: "mr-2 flex items-center text-muted-foreground/70 shrink-0", children: startIcon }) : isSearch ? /* @__PURE__ */ jsx8(Search, { className: "mr-2 h-3.5 w-3.5 text-muted-foreground/70 shrink-0" }) : null,
          /* @__PURE__ */ jsx8(
            "input",
            {
              ...props,
              ref,
              type,
              value,
              placeholder: resolvedPlaceholder,
              className: cn(
                "w-full bg-transparent text-xs text-brand-snow placeholder:text-muted-foreground/60 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 [color-scheme:dark]",
                className
              )
            }
          ),
          onClear && value && String(value).length > 0 ? /* @__PURE__ */ jsx8(
            "button",
            {
              type: "button",
              onClick: onClear,
              className: "ml-1 text-muted-foreground/60 hover:text-ink transition-colors shrink-0",
              children: /* @__PURE__ */ jsx8(X, { className: "h-3.5 w-3.5" })
            }
          ) : endIcon ? /* @__PURE__ */ jsx8("span", { className: "ml-2 flex items-center text-muted-foreground/70 shrink-0", children: endIcon }) : null
        ]
      }
    );
  }
);
Input.displayName = "Input";

// src/ui/select.tsx
import * as React5 from "react";
import * as SelectPrimitive from "@radix-ui/react-select";
import { CheckIcon, ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { jsx as jsx9, jsxs as jsxs5 } from "react/jsx-runtime";
var Select = SelectPrimitive.Root;
var SelectGroup = SelectPrimitive.Group;
var SelectValue = SelectPrimitive.Value;
var SelectTrigger = React5.forwardRef(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs5(
  SelectPrimitive.Trigger,
  {
    ref,
    className: cn(
      "flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-ink/15 bg-background-800/90 px-3 py-1.5 text-xs text-brand-snow shadow-sm ring-offset-background data-[placeholder]:text-muted-foreground/60 transition-colors focus:outline-none focus:ring-1 focus:ring-brand-sky/60 focus:border-brand-sky/60 disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
      className
    ),
    ...props,
    children: [
      children,
      /* @__PURE__ */ jsx9(SelectPrimitive.Icon, { asChild: true, children: /* @__PURE__ */ jsx9(ChevronDownIcon, { className: "h-4 w-4 opacity-50" }) })
    ]
  }
));
SelectTrigger.displayName = SelectPrimitive.Trigger.displayName;
var SelectScrollUpButton = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx9(
  SelectPrimitive.ScrollUpButton,
  {
    ref,
    className: cn("flex cursor-default items-center justify-center py-1", className),
    ...props,
    children: /* @__PURE__ */ jsx9(ChevronUpIcon, { className: "h-4 w-4" })
  }
));
SelectScrollUpButton.displayName = SelectPrimitive.ScrollUpButton.displayName;
var SelectScrollDownButton = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx9(
  SelectPrimitive.ScrollDownButton,
  {
    ref,
    className: cn("flex cursor-default items-center justify-center py-1", className),
    ...props,
    children: /* @__PURE__ */ jsx9(ChevronDownIcon, { className: "h-4 w-4" })
  }
));
SelectScrollDownButton.displayName = SelectPrimitive.ScrollDownButton.displayName;
var SelectContent = React5.forwardRef(({ className, children, position = "popper", ...props }, ref) => /* @__PURE__ */ jsx9(SelectPrimitive.Portal, { children: /* @__PURE__ */ jsxs5(
  SelectPrimitive.Content,
  {
    ref,
    className: cn(
      "relative z-50 max-h-[--radix-select-content-available-height] min-w-[8rem] overflow-y-auto overflow-x-hidden rounded-md border border-ink/15 bg-background-800 text-brand-snow shadow-xl backdrop-blur-md data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 origin-[--radix-select-content-transform-origin]",
      position === "popper" && "data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1",
      className
    ),
    position,
    ...props,
    children: [
      /* @__PURE__ */ jsx9(SelectScrollUpButton, {}),
      /* @__PURE__ */ jsx9(
        SelectPrimitive.Viewport,
        {
          className: cn(
            "p-1",
            position === "popper" && "h-[var(--radix-select-trigger-height)] w-full min-w-[var(--radix-select-trigger-width)]"
          ),
          children
        }
      ),
      /* @__PURE__ */ jsx9(SelectScrollDownButton, {})
    ]
  }
) }));
SelectContent.displayName = SelectPrimitive.Content.displayName;
var SelectLabel = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx9(
  SelectPrimitive.Label,
  {
    ref,
    className: cn("px-2 py-1.5 text-sm font-semibold", className),
    ...props
  }
));
SelectLabel.displayName = SelectPrimitive.Label.displayName;
var SelectItem = React5.forwardRef(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs5(
  SelectPrimitive.Item,
  {
    ref,
    className: cn(
      "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none focus:bg-background-400 focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className
    ),
    ...props,
    children: [
      /* @__PURE__ */ jsx9("span", { className: "absolute right-2 flex h-3.5 w-3.5 items-center justify-center", children: /* @__PURE__ */ jsx9(SelectPrimitive.ItemIndicator, { children: /* @__PURE__ */ jsx9(CheckIcon, { className: "h-4 w-4" }) }) }),
      /* @__PURE__ */ jsx9(SelectPrimitive.ItemText, { children })
    ]
  }
));
SelectItem.displayName = SelectPrimitive.Item.displayName;
var SelectSeparator = React5.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx9(
  SelectPrimitive.Separator,
  {
    ref,
    className: cn("-mx-1 my-1 h-px bg-muted", className),
    ...props
  }
));
SelectSeparator.displayName = SelectPrimitive.Separator.displayName;

// src/ui/skeleton.tsx
import { jsx as jsx10 } from "react/jsx-runtime";
function Skeleton({ className, ...props }) {
  return /* @__PURE__ */ jsx10("div", { className: cn("animate-pulse rounded-md bg-primary/10", className), ...props });
}

// src/ui/table.tsx
import * as React6 from "react";
import { jsx as jsx11 } from "react/jsx-runtime";
var Table = React6.forwardRef(({ className, containerClassName, ...props }, ref) => /* @__PURE__ */ jsx11(
  "div",
  {
    className: cn(
      "relative w-full overflow-auto rounded-xl border border-ink/10 bg-background-800/50 backdrop-blur-sm shadow-sm",
      containerClassName
    ),
    children: /* @__PURE__ */ jsx11(
      "table",
      {
        ref,
        className: cn("w-full caption-bottom text-xs text-foreground-200 border-collapse", className),
        ...props
      }
    )
  }
));
Table.displayName = "Table";
var TableHeader = React6.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "thead",
  {
    ref,
    className: cn("bg-ink/[0.03] border-b border-ink/10", className),
    ...props
  }
));
TableHeader.displayName = "TableHeader";
var TableBody = React6.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "tbody",
  {
    ref,
    className: cn("[&_tr:last-child]:border-0 divide-y divide-ink/[0.05]", className),
    ...props
  }
));
TableBody.displayName = "TableBody";
var TableFooter = React6.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "tfoot",
  {
    ref,
    className: cn("border-t border-ink/10 bg-ink/[0.02] font-medium text-xs", className),
    ...props
  }
));
TableFooter.displayName = "TableFooter";
var TableRow = React6.forwardRef(
  ({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
    "tr",
    {
      ref,
      className: cn(
        "border-b border-ink/[0.06] transition-colors hover:bg-ink/[0.03] data-[state=selected]:bg-ink/[0.06]",
        className
      ),
      ...props
    }
  )
);
TableRow.displayName = "TableRow";
var TableHead = React6.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "th",
  {
    ref,
    className: cn(
      "h-9 px-3.5 text-left align-middle font-semibold text-muted-foreground uppercase tracking-wider text-xs select-none",
      className
    ),
    ...props
  }
));
TableHead.displayName = "TableHead";
var TableCell = React6.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11(
  "td",
  {
    ref,
    className: cn("px-3.5 py-3 align-middle text-xs text-foreground-200", className),
    ...props
  }
));
TableCell.displayName = "TableCell";
var TableCaption = React6.forwardRef(({ className, ...props }, ref) => /* @__PURE__ */ jsx11("caption", { ref, className: cn("mt-4 text-xs text-muted-foreground", className), ...props }));
TableCaption.displayName = "TableCaption";

// src/layout/RouteTabs.tsx
import { jsx as jsx12, jsxs as jsxs6 } from "react/jsx-runtime";
var tabListClass = "flex w-full items-end gap-1 border-b border-ink/10 overflow-x-auto";
var tabTriggerClass = cn(
  "relative -mb-px inline-flex items-center gap-2 whitespace-nowrap rounded-t-md px-4 py-2.5",
  "text-sm font-medium text-ink/55 border-b-2 border-transparent transition-colors",
  "hover:text-ink/90 hover:bg-ink/[0.03]",
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-sky/50"
);
var tabActiveClass = "text-ink font-semibold border-brand-sky bg-brand-sky/[0.08]";
function RouteTabs({
  tabs,
  activePath,
  className
}) {
  const Link = useLink();
  const ctxPath = useActivePath();
  const pathname = (activePath ?? ctxPath).replace(/\/$/, "");
  return /* @__PURE__ */ jsx12("nav", { className: cn(tabListClass, "mb-6", className), role: "tablist", children: tabs.map(({ label, to, icon: Icon2, count }) => {
    const cleanTo = to.replace(/\/$/, "");
    const active = pathname.endsWith(cleanTo) || pathname.includes(`${cleanTo}/`);
    return /* @__PURE__ */ jsxs6(
      Link,
      {
        to,
        role: "tab",
        "aria-selected": active,
        className: cn(tabTriggerClass, active && tabActiveClass),
        children: [
          Icon2 && /* @__PURE__ */ jsx12(Icon2, { className: "h-4 w-4" }),
          label,
          /* @__PURE__ */ jsx12(CountBadge, { n: count, showZero: true })
        ]
      },
      to
    );
  }) });
}

// src/layout/FieldGrid.tsx
import { jsx as jsx13, jsxs as jsxs7 } from "react/jsx-runtime";
var COLS = {
  1: "",
  2: "md:grid-cols-2",
  3: "md:grid-cols-3",
  4: "md:grid-cols-2 xl:grid-cols-4"
};
var SPAN = { 1: "", 2: "md:col-span-2", 3: "md:col-span-3", 4: "md:col-span-full" };
function FieldGrid({
  fields,
  columns = 3,
  className
}) {
  const visible = fields.filter((f) => !f.hidden);
  return (
    // Each cell draws its right and bottom border; the frame clips the outer ones.
    /* @__PURE__ */ jsx13("div", { className: cn("overflow-hidden rounded-lg border border-ink/10", className), children: /* @__PURE__ */ jsx13("dl", { className: cn("-mb-px -mr-px grid grid-cols-1", COLS[Math.min(columns, visible.length)]), children: visible.map((f) => /* @__PURE__ */ jsxs7(
      "div",
      {
        className: cn("min-w-0 border-b border-r border-ink/10 px-4 py-3", SPAN[f.span ?? 1]),
        children: [
          /* @__PURE__ */ jsx13("dt", { className: "mb-1 text-xs font-medium text-ink/50", children: f.label }),
          /* @__PURE__ */ jsx13("dd", { className: "min-w-0 text-sm text-ink/90", children: f.value ?? /* @__PURE__ */ jsx13("span", { className: "text-ink/35", children: "\u2014" }) })
        ]
      },
      f.label
    )) }) })
  );
}

// src/layout/PageHeader.tsx
import { jsx as jsx14, jsxs as jsxs8 } from "react/jsx-runtime";
function PageHeader({
  title,
  badge,
  eyebrow,
  description,
  actions,
  className,
  children,
  ...props
}) {
  return /* @__PURE__ */ jsxs8(
    "header",
    {
      className: cn(
        "pt-4 mb-5 pb-4 border-b border-ink/10 sticky top-12 z-30 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 transition-colors",
        className
      ),
      ...props,
      children: [
        /* @__PURE__ */ jsxs8("div", { className: "flex flex-wrap items-start justify-between gap-x-6 gap-y-3", children: [
          /* @__PURE__ */ jsxs8("div", { className: "min-w-0 flex-1", children: [
            eyebrow && /* @__PURE__ */ jsx14("div", { className: "mb-1 text-xs font-medium uppercase tracking-wide text-ink/50", children: eyebrow }),
            /* @__PURE__ */ jsxs8(
              heading_default,
              {
                level: "h1",
                className: "flex flex-wrap gap-2 items-center font-display text-2xl mb-0 text-ink/90",
                children: [
                  /* @__PURE__ */ jsx14("span", { className: "min-w-0 break-words", children: title }),
                  badge
                ]
              }
            ),
            description && /* @__PURE__ */ jsx14("p", { className: "mt-1 text-sm text-ink/60", children: description })
          ] }),
          actions && /* @__PURE__ */ jsx14("div", { className: "flex shrink-0 flex-wrap items-center gap-2", children: actions })
        ] }),
        children
      ]
    }
  );
}

// src/layout/PageLayout.tsx
import { jsx as jsx15 } from "react/jsx-runtime";
function PageLayout({ className, children, ...props }) {
  return /* @__PURE__ */ jsx15("div", { className: cn("space-y-4 pb-8 px-3 w-full", className), ...props, children });
}

// src/layout/PageSection.tsx
import { jsx as jsx16, jsxs as jsxs9 } from "react/jsx-runtime";
function PageSection({ title, action, className, children, ...props }) {
  return /* @__PURE__ */ jsxs9("div", { className: cn("mb-6", className), ...props, children: [
    /* @__PURE__ */ jsxs9("div", { className: "flex items-center justify-between gap-4 mb-2", children: [
      title && /* @__PURE__ */ jsx16(
        heading_default,
        {
          level: "h6",
          className: "mt-2 text-ink/80 uppercase tracking-wide text-xs font-semibold",
          children: title
        }
      ),
      action && /* @__PURE__ */ jsx16("div", { className: "mt-1 ml-auto", children: action })
    ] }),
    children
  ] });
}

// src/table/DataTable.tsx
import { useState as useState2, useMemo as useMemo2, useEffect as useEffect2, useCallback as useCallback2 } from "react";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Inbox,
  SearchX,
  X as X2
} from "lucide-react";
import { Fragment as Fragment2, jsx as jsx17, jsxs as jsxs10 } from "react/jsx-runtime";
var toSnakeCase = (str) => str.replace(/[A-Z]/g, (l) => `_${l.toLowerCase()}`);
var isSortConfigMatch = (configKey, colKey) => {
  if (!configKey) return false;
  return configKey === colKey || configKey === toSnakeCase(colKey);
};
function DataTable({
  columns,
  data,
  keyExtractor,
  onRowClick,
  className,
  containerClassName,
  emptyMessage,
  emptyDescription,
  title,
  subtitle,
  toolbarActions,
  searchable = true,
  searchPlaceholder = "Filter records...",
  filterFn,
  filters,
  defaultSortKey,
  defaultSortDirection,
  paginated = true,
  pageSize = 10,
  pageSizeOptions = [10, 20, 50, 100],
  hideToolbar = false,
  hideCount = false,
  serverSide = false,
  totalCount,
  nextCursor,
  queryParams,
  onQueryChange,
  syncUrlParams = true,
  loading = false
}) {
  const rawItems = useMemo2(() => {
    if (!data) return [];
    if (Array.isArray(data)) return data;
    if (typeof data === "object" && Array.isArray(data.items)) {
      return data.items;
    }
    return [];
  }, [data]);
  const totalCountValue = useMemo2(() => {
    if (totalCount !== void 0) return totalCount;
    if (data && typeof data === "object" && typeof data.total === "number") {
      return data.total;
    }
    return void 0;
  }, [totalCount, data]);
  const isServerDriven = serverSide || Boolean(onQueryChange);
  const [searchQuery, setSearchQuery] = useState2(queryParams?.search ?? "");
  useEffect2(() => {
    if (queryParams?.search !== void 0 && queryParams.search !== searchQuery) {
      setSearchQuery(queryParams.search);
    }
  }, [queryParams?.search]);
  const [sortConfig, setSortConfig] = useState2(() => {
    if (queryParams?.sort) {
      const s = queryParams.sort;
      if (s.endsWith("_asc")) return { key: s.slice(0, -4), direction: "asc" };
      if (s.endsWith("_desc")) return { key: s.slice(0, -5), direction: "desc" };
    }
    return defaultSortKey ? { key: defaultSortKey, direction: defaultSortDirection ?? "asc" } : null;
  });
  useEffect2(() => {
    if (queryParams?.sort) {
      const s = queryParams.sort;
      if (s.endsWith("_asc")) {
        const key = s.slice(0, -4);
        setSortConfig({ key, direction: "asc" });
      } else if (s.endsWith("_desc")) {
        const key = s.slice(0, -5);
        setSortConfig({ key, direction: "desc" });
      }
    }
  }, [queryParams?.sort]);
  const [currentPage, setCurrentPage] = useState2(queryParams?.page ?? 1);
  useEffect2(() => {
    if (queryParams?.page !== void 0) {
      setCurrentPage(queryParams.page);
    }
  }, [queryParams?.page]);
  const [itemsPerPage, setItemsPerPage] = useState2(() => {
    if (!paginated) return void 0;
    const initialLimit = queryParams?.limit ?? pageSize;
    if (initialLimit <= 0 || initialLimit === Infinity) return void 0;
    return initialLimit;
  });
  useEffect2(() => {
    if (!paginated) {
      setItemsPerPage(void 0);
    } else if (queryParams?.limit !== void 0) {
      setItemsPerPage(queryParams.limit <= 0 ? void 0 : queryParams.limit);
    } else if (pageSize <= 0 || pageSize === Infinity) {
      setItemsPerPage(void 0);
    } else {
      setItemsPerPage(pageSize);
    }
  }, [paginated, pageSize, queryParams?.limit]);
  const [filterValues, setFilterValues] = useState2(() => {
    const initial = {};
    filters?.forEach((f) => {
      initial[f.id] = queryParams?.filters?.[f.id] ?? f.value ?? f.defaultValue ?? "all";
    });
    return initial;
  });
  useEffect2(() => {
    if (filters || queryParams?.filters) {
      setFilterValues((prev) => {
        let changed = false;
        const next = { ...prev };
        filters?.forEach((f) => {
          const targetVal = queryParams?.filters?.[f.id] ?? f.value ?? f.defaultValue ?? "all";
          if (prev[f.id] !== targetVal) {
            next[f.id] = targetVal;
            changed = true;
          }
        });
        return changed ? next : prev;
      });
    }
  }, [filters, queryParams?.filters]);
  const handleFilterChange = useCallback2(
    (filter, val) => {
      setFilterValues((prev) => {
        const next = { ...prev, [filter.id]: val };
        if (onQueryChange) {
          onQueryChange({ filters: next, page: 1 });
        }
        return next;
      });
      setCurrentPage(1);
      if (filter.onChange) {
        filter.onChange(val);
      }
    },
    [onQueryChange]
  );
  const handleSearchChange = useCallback2(
    (val) => {
      setSearchQuery(val);
      setCurrentPage(1);
      if (onQueryChange) {
        onQueryChange({ search: val, page: 1 });
      }
    },
    [onQueryChange]
  );
  const handlePageSizeChange = useCallback2(
    (newLimit) => {
      setItemsPerPage(newLimit === -1 ? void 0 : newLimit);
      setCurrentPage(1);
      if (onQueryChange) {
        onQueryChange({ limit: newLimit === -1 ? 100 : newLimit, page: 1 });
      }
    },
    [onQueryChange]
  );
  const handlePageChange = useCallback2(
    (newPage) => {
      setCurrentPage(newPage);
      if (onQueryChange) {
        onQueryChange({ page: newPage });
      }
    },
    [onQueryChange]
  );
  const filteredData = useMemo2(() => {
    if (isServerDriven) return rawItems;
    let result = rawItems;
    if (filters && filters.length > 0) {
      result = result.filter((item) => {
        for (const f of filters) {
          const selectedVal = f.value !== void 0 ? f.value : filterValues[f.id] ?? "all";
          if (!selectedVal || selectedVal === "all") continue;
          if (f.filterFn) {
            if (!f.filterFn(item, selectedVal)) return false;
          } else if (f.accessorKey) {
            const raw = item[f.accessorKey];
            if (raw === null || raw === void 0) return false;
            if (String(raw).toLowerCase() !== selectedVal.toLowerCase()) {
              return false;
            }
          }
        }
        return true;
      });
    }
    const trimmed = searchQuery.trim().toLowerCase();
    if (!trimmed) return result;
    const terms = trimmed.split(/\s+/).filter(Boolean);
    return result.filter((item) => {
      if (filterFn) return filterFn(item, trimmed);
      for (const col of columns) {
        if (col.searchable === false) continue;
        let valStr = "";
        if (col.searchValue) {
          valStr = col.searchValue(item).toLowerCase();
        } else if (col.accessorKey) {
          const raw = item[col.accessorKey];
          if (raw !== null && raw !== void 0) {
            valStr = String(raw).toLowerCase();
          }
        }
        if (valStr && terms.every((t) => valStr.includes(t))) {
          return true;
        }
      }
      const itemValues = Object.values(item).filter((v) => typeof v === "string" || typeof v === "number").map((v) => String(v).toLowerCase()).join(" ");
      return terms.every((t) => itemValues.includes(t));
    });
  }, [rawItems, isServerDriven, filters, filterValues, searchQuery, filterFn, columns]);
  const sortedData = useMemo2(() => {
    if (isServerDriven) return filteredData;
    if (!sortConfig) return filteredData;
    const col = columns.find((c, index) => {
      const k = c.sortKey ?? c.accessorKey ?? (typeof c.header === "string" ? c.header : `col_${index}`);
      return isSortConfigMatch(sortConfig.key, String(k));
    });
    return [...filteredData].sort((a, b) => {
      let aVal;
      let bVal;
      if (col?.sortValue) {
        aVal = col.sortValue(a);
        bVal = col.sortValue(b);
      } else if (col?.accessorKey) {
        aVal = a[col.accessorKey];
        bVal = b[col.accessorKey];
      } else {
        aVal = a[sortConfig.key];
        bVal = b[sortConfig.key];
      }
      if (aVal === bVal) return 0;
      if (aVal === null || aVal === void 0) return sortConfig.direction === "asc" ? 1 : -1;
      if (bVal === null || bVal === void 0) return sortConfig.direction === "asc" ? -1 : 1;
      if (aVal instanceof Date && bVal instanceof Date) {
        return sortConfig.direction === "asc" ? aVal.getTime() - bVal.getTime() : bVal.getTime() - aVal.getTime();
      }
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortConfig.direction === "asc" ? aVal - bVal : bVal - aVal;
      }
      if (typeof aVal === "boolean" && typeof bVal === "boolean") {
        return sortConfig.direction === "asc" ? aVal === bVal ? 0 : aVal ? 1 : -1 : aVal === bVal ? 0 : aVal ? -1 : 1;
      }
      if (typeof aVal === "string" && typeof bVal === "string") {
        const isDateLike = /^\d{4}-\d{2}-\d{2}/.test(aVal) && /^\d{4}-\d{2}-\d{2}/.test(bVal);
        if (isDateLike) {
          const aTime = Date.parse(aVal);
          const bTime = Date.parse(bVal);
          if (!isNaN(aTime) && !isNaN(bTime)) {
            return sortConfig.direction === "asc" ? aTime - bTime : bTime - aTime;
          }
        }
      }
      const aStr = String(aVal).toLowerCase();
      const bStr = String(bVal).toLowerCase();
      return sortConfig.direction === "asc" ? aStr.localeCompare(bStr) : bStr.localeCompare(aStr);
    });
  }, [filteredData, isServerDriven, sortConfig, columns]);
  const activeItemsPerPage = itemsPerPage ?? (isServerDriven ? rawItems.length || 10 : void 0);
  const hasMore = isServerDriven ? totalCountValue !== void 0 ? currentPage < Math.ceil(totalCountValue / (activeItemsPerPage || 10)) : activeItemsPerPage ? rawItems.length === activeItemsPerPage : false : safeCurrentPageCheck();
  function safeCurrentPageCheck() {
    const calcPages = activeItemsPerPage ? Math.ceil(sortedData.length / activeItemsPerPage) || 1 : 1;
    return currentPage < calcPages;
  }
  const totalPages = totalCountValue !== void 0 ? activeItemsPerPage ? Math.ceil(totalCountValue / activeItemsPerPage) || 1 : 1 : isServerDriven ? hasMore ? currentPage + 1 : Math.max(1, currentPage) : activeItemsPerPage ? Math.ceil(sortedData.length / activeItemsPerPage) || 1 : 1;
  const safeCurrentPage = totalCountValue !== void 0 || !isServerDriven ? Math.min(Math.max(1, currentPage), totalPages) : Math.max(1, currentPage);
  const totalItems = totalCountValue !== void 0 ? totalCountValue : isServerDriven ? (safeCurrentPage - 1) * (activeItemsPerPage || 10) + rawItems.length : sortedData.length;
  useEffect2(() => {
    if ((totalCountValue !== void 0 || !isServerDriven) && currentPage > totalPages) {
      setCurrentPage(totalPages);
    }
  }, [currentPage, totalPages, totalCountValue, isServerDriven]);
  const paginatedData = useMemo2(() => {
    if (isServerDriven) return rawItems;
    if (!itemsPerPage) return sortedData;
    const start = (safeCurrentPage - 1) * itemsPerPage;
    return sortedData.slice(start, start + itemsPerPage);
  }, [isServerDriven, rawItems, sortedData, safeCurrentPage, itemsPerPage]);
  const startItem = rawItems.length === 0 ? 0 : (safeCurrentPage - 1) * (activeItemsPerPage ?? (isServerDriven ? rawItems.length : totalItems)) + 1;
  const endItem = rawItems.length === 0 ? 0 : isServerDriven && totalCountValue === void 0 ? startItem + rawItems.length - 1 : activeItemsPerPage ? Math.min(safeCurrentPage * activeItemsPerPage, totalItems) : totalItems;
  const isColumnSortable = (col) => {
    if (col.sortable === false) return false;
    const key = col.sortKey ?? col.accessorKey;
    return key !== void 0 && toSnakeCase(String(key)) === "created_at";
  };
  const handleHeaderClick = (col, index) => {
    if (!isColumnSortable(col)) return;
    const colKey = col.sortKey ?? col.accessorKey ?? (typeof col.header === "string" ? col.header : `col_${index}`);
    const colKeyStr = String(colKey);
    const colSnakeKey = toSnakeCase(colKeyStr);
    const isCurrentKey = isSortConfigMatch(sortConfig?.key, colKeyStr);
    const nextDirection = isCurrentKey && sortConfig?.direction === "asc" ? "desc" : "asc";
    setSortConfig({ key: colSnakeKey, direction: nextDirection });
    if (onQueryChange) {
      const backendSort = mapToSortParam(colKeyStr, nextDirection);
      onQueryChange({ sort: backendSort, page: 1 });
    }
  };
  const hasActiveFilter = Boolean(
    filters?.some((f) => {
      const val = f.value !== void 0 ? f.value : filterValues[f.id];
      return val && val !== "all";
    })
  );
  const isFiltered = searchQuery.trim().length > 0 || hasActiveFilter;
  const showToolbar = !hideToolbar && (searchable || title || toolbarActions || !hideCount || Boolean(filters && filters.length > 0));
  const handleResetAll = useCallback2(() => {
    setSearchQuery("");
    const resetVals = {};
    if (filters) {
      filters.forEach((f) => {
        resetVals[f.id] = "all";
        if (f.onChange) f.onChange("all");
      });
      setFilterValues(resetVals);
    }
    setCurrentPage(1);
    if (onQueryChange) {
      onQueryChange({ search: "", filters: resetVals, page: 1 });
    }
  }, [filters, onQueryChange]);
  return /* @__PURE__ */ jsxs10("div", { className: "flex flex-col gap-2.5 w-full", children: [
    showToolbar && /* @__PURE__ */ jsxs10("div", { className: "flex flex-col sm:flex-row sm:items-center justify-between gap-3 px-1 py-1", children: [
      /* @__PURE__ */ jsxs10("div", { className: "flex flex-wrap items-center gap-3", children: [
        title && /* @__PURE__ */ jsxs10("div", { className: "flex flex-col", children: [
          /* @__PURE__ */ jsx17("h3", { className: "text-sm font-semibold text-brand-snow", children: title }),
          subtitle && /* @__PURE__ */ jsx17("p", { className: "text-xs text-muted-foreground", children: subtitle })
        ] }),
        searchable && /* @__PURE__ */ jsx17("div", { className: "w-full sm:w-64 max-w-sm", children: /* @__PURE__ */ jsx17(
          Input,
          {
            type: "search",
            value: searchQuery,
            onChange: (e) => handleSearchChange(e.target.value),
            onClear: () => handleSearchChange(""),
            placeholder: searchPlaceholder,
            className: "text-xs"
          }
        ) }),
        filters && filters.length > 0 && /* @__PURE__ */ jsx17("div", { className: "flex flex-wrap items-center gap-2", children: filters.map((f) => {
          const currentVal = f.value !== void 0 ? f.value : filterValues[f.id] ?? "all";
          return /* @__PURE__ */ jsx17("div", { className: "flex items-center gap-1.5", children: /* @__PURE__ */ jsxs10(Select, { value: currentVal, onValueChange: (v) => handleFilterChange(f, v), children: [
            /* @__PURE__ */ jsxs10(
              SelectTrigger,
              {
                "aria-label": f.label,
                className: cn(
                  "h-8 w-auto gap-2",
                  currentVal !== "all" && "border-brand-sky/60 bg-brand-sky/10 text-brand-sky font-medium"
                ),
                children: [
                  /* @__PURE__ */ jsxs10("span", { className: "text-ink/50", children: [
                    f.label,
                    ":"
                  ] }),
                  /* @__PURE__ */ jsx17(SelectValue, {})
                ]
              }
            ),
            /* @__PURE__ */ jsxs10(SelectContent, { children: [
              /* @__PURE__ */ jsx17(SelectItem, { value: "all", children: "All" }),
              f.options.map((opt) => /* @__PURE__ */ jsx17(SelectItem, { value: opt.value, children: opt.label }, opt.value))
            ] })
          ] }) }, f.id);
        }) }),
        !hideCount && (isFiltered || !paginated) && /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-1.5", children: [
          isFiltered ? /* @__PURE__ */ jsxs10(Badge, { variant: "info", size: "sm", className: "text-xs", children: [
            rawItems.length,
            " of ",
            totalItems
          ] }) : /* @__PURE__ */ jsxs10("span", { className: "text-xs text-ink/50", children: [
            totalItems,
            " ",
            totalItems === 1 ? "record" : "records"
          ] }),
          isFiltered && /* @__PURE__ */ jsxs10(
            Button,
            {
              variant: "ghost",
              size: "xs",
              onClick: handleResetAll,
              className: "text-muted-foreground hover:text-ink gap-1",
              children: [
                /* @__PURE__ */ jsx17(X2, { className: "h-3 w-3" }),
                "Clear"
              ]
            }
          )
        ] })
      ] }),
      toolbarActions && /* @__PURE__ */ jsx17("div", { className: "flex items-center gap-2 shrink-0", children: toolbarActions })
    ] }),
    /* @__PURE__ */ jsxs10(Table, { className, containerClassName, children: [
      /* @__PURE__ */ jsx17(TableHeader, { className: "sticky top-0 z-10 bg-background-800/90 backdrop-blur border-b border-ink/10", children: /* @__PURE__ */ jsx17(TableRow, { children: columns.map((col, index) => {
        const sortable = isColumnSortable(col);
        const colKey = col.sortKey ?? col.accessorKey ?? (typeof col.header === "string" ? col.header : `col_${index}`);
        const colKeyStr = String(colKey);
        const isSortedAsc = isSortConfigMatch(sortConfig?.key, colKeyStr) && sortConfig?.direction === "asc";
        const isSortedDesc = isSortConfigMatch(sortConfig?.key, colKeyStr) && sortConfig?.direction === "desc";
        return /* @__PURE__ */ jsx17(
          TableHead,
          {
            className: cn(
              sortable && "cursor-pointer select-none hover:text-ink transition-colors group",
              col.className,
              col.headerClassName
            ),
            onClick: () => handleHeaderClick(col, index),
            children: /* @__PURE__ */ jsxs10("div", { className: "inline-flex items-center gap-1.5", children: [
              /* @__PURE__ */ jsx17("span", { children: col.header }),
              sortable && /* @__PURE__ */ jsx17("span", { className: "shrink-0", children: isSortedAsc ? /* @__PURE__ */ jsx17(ArrowUp, { className: "h-3.5 w-3.5 text-brand-sky" }) : isSortedDesc ? /* @__PURE__ */ jsx17(ArrowDown, { className: "h-3.5 w-3.5 text-brand-sky" }) : /* @__PURE__ */ jsx17(ArrowUpDown, { className: "h-3.5 w-3.5 opacity-25 group-hover:opacity-70 transition-opacity" }) })
            ] })
          },
          index
        );
      }) }) }),
      /* @__PURE__ */ jsx17(
        TableBody,
        {
          className: cn("transition-opacity duration-200", loading && "opacity-50 pointer-events-none"),
          children: paginatedData.length === 0 ? /* @__PURE__ */ jsx17(TableRow, { className: "hover:bg-transparent", children: /* @__PURE__ */ jsx17(TableCell, { colSpan: columns.length, className: "text-center py-12", children: isFiltered ? /* @__PURE__ */ jsxs10("div", { className: "flex flex-col items-center justify-center gap-2", children: [
            /* @__PURE__ */ jsx17("div", { className: "p-2.5 rounded-full bg-ink/5 border border-ink/10 text-muted-foreground", children: /* @__PURE__ */ jsx17(SearchX, { className: "h-5 w-5" }) }),
            /* @__PURE__ */ jsx17("span", { className: "text-xs font-medium text-foreground", children: "No matching records found" }),
            /* @__PURE__ */ jsx17("span", { className: "text-xs text-muted-foreground", children: "No items matched current search or filter criteria" }),
            /* @__PURE__ */ jsx17(
              Button,
              {
                variant: "outline",
                size: "xs",
                onClick: handleResetAll,
                className: "mt-2",
                children: "Reset filter"
              }
            )
          ] }) : /* @__PURE__ */ jsxs10("div", { className: "flex flex-col items-center justify-center gap-2", children: [
            /* @__PURE__ */ jsx17("div", { className: "p-2.5 rounded-full bg-ink/5 border border-ink/10 text-muted-foreground/50", children: /* @__PURE__ */ jsx17(Inbox, { className: "h-5 w-5" }) }),
            /* @__PURE__ */ jsx17("span", { className: "text-xs text-muted-foreground italic", children: emptyMessage ?? "Nothing here yet" }),
            emptyDescription && /* @__PURE__ */ jsx17("span", { className: "text-xs text-muted-foreground/70 max-w-md", children: emptyDescription })
          ] }) }) }) : paginatedData.map((item, itemIndex) => /* @__PURE__ */ jsx17(
            TableRow,
            {
              onClick: () => onRowClick && onRowClick(item),
              className: cn(
                onRowClick && "cursor-pointer hover:bg-ink/[0.04] active:bg-ink/[0.06] transition-colors"
              ),
              children: columns.map((col, index) => /* @__PURE__ */ jsx17(
                TableCell,
                {
                  className: cn("text-ink [&>p]:text-ink", col.className),
                  children: col.cell ? col.cell(item) : item[col.accessorKey]
                },
                index
              ))
            },
            keyExtractor ? keyExtractor(item) || `row_${itemIndex}` : `row_${itemIndex}`
          ))
        }
      )
    ] }),
    paginated && (totalItems > 0 || safeCurrentPage > 1) && /* @__PURE__ */ jsxs10("div", { className: "flex flex-col sm:flex-row items-center justify-between gap-3 px-3 py-2 text-xs text-muted-foreground border-t border-ink/10 bg-background-800/30 rounded-b-lg", children: [
      /* @__PURE__ */ jsx17("div", { className: "flex items-center gap-1.5", children: /* @__PURE__ */ jsxs10("span", { children: [
        "Showing ",
        /* @__PURE__ */ jsx17("span", { className: "font-medium text-brand-snow", children: startItem }),
        " to",
        " ",
        /* @__PURE__ */ jsx17("span", { className: "font-medium text-brand-snow", children: endItem }),
        totalCountValue !== void 0 ? /* @__PURE__ */ jsxs10(Fragment2, { children: [
          " of ",
          /* @__PURE__ */ jsx17("span", { className: "font-medium text-brand-snow", children: totalItems }),
          " ",
          totalItems === 1 ? "record" : "records"
        ] }) : hasMore ? /* @__PURE__ */ jsxs10(Fragment2, { children: [
          " of ",
          /* @__PURE__ */ jsxs10("span", { className: "font-medium text-brand-snow", children: [
            endItem,
            "+"
          ] }),
          " records"
        ] }) : " records"
      ] }) }),
      /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-4 flex-wrap", children: [
        /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-1.5", children: [
          /* @__PURE__ */ jsx17("span", { className: "text-xs text-muted-foreground", children: "Rows per page:" }),
          /* @__PURE__ */ jsxs10(
            Select,
            {
              value: String(activeItemsPerPage ?? -1),
              onValueChange: (v) => handlePageSizeChange(Number(v)),
              children: [
                /* @__PURE__ */ jsx17(SelectTrigger, { "aria-label": "Rows per page", className: "h-7 w-auto gap-2", children: /* @__PURE__ */ jsx17(SelectValue, {}) }),
                /* @__PURE__ */ jsxs10(SelectContent, { children: [
                  pageSizeOptions.map((opt) => /* @__PURE__ */ jsx17(SelectItem, { value: String(opt), children: opt }, opt)),
                  /* @__PURE__ */ jsx17(SelectItem, { value: "-1", children: "All" })
                ] })
              ]
            }
          )
        ] }),
        /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-1", children: [
          /* @__PURE__ */ jsxs10("span", { className: "text-xs mr-1", children: [
            "Page ",
            /* @__PURE__ */ jsx17("span", { className: "font-medium text-brand-snow", children: safeCurrentPage }),
            totalCountValue !== void 0 ? /* @__PURE__ */ jsxs10(Fragment2, { children: [
              " of ",
              /* @__PURE__ */ jsx17("span", { className: "font-medium text-brand-snow", children: totalPages })
            ] }) : hasMore ? null : /* @__PURE__ */ jsxs10(Fragment2, { children: [
              " of ",
              /* @__PURE__ */ jsx17("span", { className: "font-medium text-brand-snow", children: safeCurrentPage })
            ] })
          ] }),
          /* @__PURE__ */ jsx17(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: safeCurrentPage <= 1,
              onClick: () => handlePageChange(1),
              className: "h-7 w-7 p-0",
              title: "First page",
              children: /* @__PURE__ */ jsx17(ChevronsLeft, { className: "h-3.5 w-3.5" })
            }
          ),
          /* @__PURE__ */ jsx17(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: safeCurrentPage <= 1,
              onClick: () => handlePageChange(Math.max(1, safeCurrentPage - 1)),
              className: "h-7 w-7 p-0",
              title: "Previous page",
              children: /* @__PURE__ */ jsx17(ChevronLeft, { className: "h-3.5 w-3.5" })
            }
          ),
          /* @__PURE__ */ jsx17(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: isServerDriven && totalCountValue === void 0 ? !hasMore : safeCurrentPage >= totalPages,
              onClick: () => handlePageChange(safeCurrentPage + 1),
              className: "h-7 w-7 p-0",
              title: "Next page",
              children: /* @__PURE__ */ jsx17(ChevronRight, { className: "h-3.5 w-3.5" })
            }
          ),
          (!isServerDriven || totalCountValue !== void 0) && /* @__PURE__ */ jsx17(
            Button,
            {
              variant: "outline",
              size: "xs",
              disabled: safeCurrentPage >= totalPages,
              onClick: () => handlePageChange(totalPages),
              className: "h-7 w-7 p-0",
              title: "Last page",
              children: /* @__PURE__ */ jsx17(ChevronsRight, { className: "h-3.5 w-3.5" })
            }
          )
        ] })
      ] })
    ] })
  ] });
}

export {
  LinkProvider,
  ActivePathProvider,
  useLink,
  useActivePath,
  NavigationProvider,
  mapToSortParam,
  useTableQueryParams,
  UNAUTHORIZED_EVENT,
  setApiGatewayBase,
  getApiGatewayBase,
  customInstance,
  badgeVariants,
  Badge,
  buttonVariants,
  Button,
  Dialog,
  DialogTrigger,
  DialogPortal,
  DialogClose,
  DialogOverlay,
  DialogContent,
  DialogHeader,
  DialogBody,
  DialogFooter,
  DialogTitle,
  DialogDescription,
  CountBadge,
  DATE_TIME_FORMAT,
  DATE_FORMAT,
  FormatDate,
  RelativeTime,
  heading_default,
  Input,
  Select,
  SelectGroup,
  SelectValue,
  SelectTrigger,
  SelectScrollUpButton,
  SelectScrollDownButton,
  SelectContent,
  SelectLabel,
  SelectItem,
  SelectSeparator,
  Skeleton,
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableRow,
  TableHead,
  TableCell,
  TableCaption,
  tabListClass,
  tabTriggerClass,
  tabActiveClass,
  RouteTabs,
  FieldGrid,
  PageHeader,
  PageLayout,
  PageSection,
  DataTable
};
//# sourceMappingURL=chunk-K5VN2DSF.js.map