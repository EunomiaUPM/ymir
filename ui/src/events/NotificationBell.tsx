import React, { useState } from "react";
import { Link, useNavigate } from "@tanstack/react-router";
import { Bell, CheckCheck } from "lucide-react";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";
import { RelativeTime } from "../ui/format-date";
import { cn } from "../lib/utils";
import { EventsApi } from "./api";
import { useLive } from "./LiveProvider";
import { EventsQueries } from "./queries";
import type { EventEnvelope, EventTopicCatalog } from "./types";

export interface NotificationBellProps {
  topics: EventTopicCatalog;
  /** The app's event feed page. */
  feedTo: string;
}

/** Header bell: unread bus events, kept current by the live connection, marked read on open. */
export const NotificationBell = ({ topics, feedTo }: NotificationBellProps) => {
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const { connected } = useLive();
  const { data: countData } = EventsQueries.useUnreadCount();
  const { data: feedData, isLoading } = EventsQueries.useFeed(
    { read: false, limit: 8, sort: "created_at_desc" },
    { enabled: open },
  );
  const { mutate: markRead } = EventsQueries.useAction(EventsApi.markRead);
  const { mutate: markAll, isPending: markingAll } = EventsQueries.useAction(
    EventsApi.markAllRead,
  );

  const count = countData?.status === 200 ? countData.data.count : 0;
  const items: EventEnvelope[] = feedData?.status === 200 ? feedData.data.items : [];

  const openEvent = (event: EventEnvelope) => {
    markRead(event.id);
    const link = topics.link(event);
    setOpen(false);
    void navigate(link ? { to: link.to as any, params: link.params as any } : { to: feedTo as any });
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          className="relative rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-ink/5 hover:text-foreground"
          aria-label={count ? `${count} unread notifications` : "Notifications"}
        >
          <Bell className="h-4 w-4" />
          {count > 0 && (
            <span className="absolute -right-1 -top-1 flex h-4 min-w-4 items-center justify-center rounded-full bg-danger-600 px-1 text-[0.625rem] font-semibold leading-none text-white tabular-nums">
              {count > 99 ? "99+" : count}
            </span>
          )}
        </button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        sideOffset={8}
        className="z-[100] w-96 overflow-hidden rounded-xl border border-ink/15 bg-background-800/95 p-0 shadow-2xl backdrop-blur-xl"
      >
        <div className="flex items-center justify-between border-b border-ink/10 px-4 py-3">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold">Notifications</span>
            <span
              className={cn("h-1.5 w-1.5 rounded-full", connected ? "bg-success-500" : "bg-ink/30")}
              title={connected ? "Live updates on" : "Live updates reconnecting"}
            />
          </div>
          <button
            type="button"
            onClick={() => markAll(undefined)}
            disabled={count === 0 || markingAll}
            className="inline-flex items-center gap-1 text-xs text-brand-sky hover:underline disabled:pointer-events-none disabled:opacity-40"
          >
            <CheckCheck className="h-3.5 w-3.5" />
            Mark all read
          </button>
        </div>

        <ul className="max-h-96 overflow-y-auto">
          {items.length === 0 && (
            <li className="px-4 py-10 text-center text-sm text-ink/50">
              {isLoading ? "Loading…" : "You are all caught up."}
            </li>
          )}
          {items.map((event) => (
            <li key={event.id} className="border-b border-ink/[0.06] last:border-0">
              <button
                type="button"
                onClick={() => openEvent(event)}
                className="flex w-full items-start gap-3 px-4 py-3 text-left hover:bg-ink/[0.04]"
              >
                <span className="mt-1.5 h-2 w-2 shrink-0 rounded-full bg-brand-sky" />
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-sm font-medium">
                    {topics.describe(event.topic)}
                  </span>
                  <span className="mt-0.5 flex items-center gap-1.5 text-xs text-ink/50">
                    <span className="truncate font-mono">{event.topic}</span>
                    <span>·</span>
                    <RelativeTime date={event.timestamp} className="shrink-0" />
                  </span>
                </span>
              </button>
            </li>
          ))}
        </ul>

        <Link
          to={feedTo as any}
          onClick={() => setOpen(false)}
          className="block border-t border-ink/10 px-4 py-2.5 text-center text-xs font-medium text-ink/70 hover:bg-ink/[0.04] hover:text-ink"
        >
          View all events
        </Link>
      </PopoverContent>
    </Popover>
  );
};
