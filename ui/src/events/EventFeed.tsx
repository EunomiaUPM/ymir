import React, { useMemo, useState } from "react";
import { Link } from "@tanstack/react-router";
import { keepPreviousData } from "@tanstack/react-query";
import { ArrowUpRight, CheckCheck } from "lucide-react";
import { useTableQueryParams } from "../hooks/useTableQueryParams";
import { DataTable } from "../table/DataTable";
import { RelativeTime } from "../ui/format-date";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import { FieldGrid } from "../layout/FieldGrid";
import { JsonTable } from "../json/JsonTable";
import { Identifier } from "../identity/Identifier";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "../ui/sheet";
import { cn } from "../lib/utils";
import { EventsApi } from "./api";
import { EventsQueries } from "./queries";
import type { EventEnvelope, EventFeedParams, EventTopicCatalog } from "./types";

const PERIODS: Record<string, number> = { hour: 3_600_000, day: 86_400_000, week: 604_800_000 };

export interface EventFeedProps {
  topics: EventTopicCatalog;
}

/** The events the user sees, filterable by area, read state and period; a row opens its payload. */
export const EventFeed = ({ topics }: EventFeedProps) => {
  const [selected, setSelected] = useState<EventEnvelope | null>(null);
  const { params, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 20,
    defaultSort: "created_at_desc",
  });

  // Table filters become the feed's query: an area is a topic pattern, a period a lower bound.
  const queryParams = useMemo<EventFeedParams>(() => {
    const { area, state, period, search: _search, ...rest } = apiParams;
    return {
      ...rest,
      ...(area && { topic: area }),
      ...(state && { read: state === "read" }),
      ...(period && { created_after: new Date(Date.now() - PERIODS[period]).toISOString() }),
    };
  }, [apiParams]);

  const { data, isFetching } = EventsQueries.useFeed(queryParams, {
    placeholderData: keepPreviousData,
  });
  const page = data?.status === 200 ? data.data : undefined;

  const { mutate: markRead } = EventsQueries.useAction(EventsApi.markRead);
  const { mutate: markAll, isPending: markingAll } = EventsQueries.useAction(
    EventsApi.markAllRead,
  );

  const open = (event: EventEnvelope) => {
    setSelected(event);
    if (!event.read_at) markRead(event.id);
  };

  return (
    <>
      <DataTable<EventEnvelope>
        data={page}
        serverSide
        loading={isFetching}
        queryParams={params}
        onQueryChange={onQueryChange}
        keyExtractor={(e) => e.id}
        onRowClick={open}
        searchable={false}
        emptyMessage="No events match these filters."
        toolbarActions={
          <Button variant="outline" size="sm" onClick={() => markAll(undefined)} isLoading={markingAll}>
            {!markingAll && <CheckCheck />}
            Mark all read
          </Button>
        }
        filters={[
          {
            id: "area",
            label: "Area",
            options: topics.areas.map((a) => ({ label: a.label, value: a.pattern })),
          },
          {
            id: "state",
            label: "State",
            options: [
              { label: "Unread", value: "unread" },
              { label: "Read", value: "read" },
            ],
          },
          {
            id: "period",
            label: "Period",
            options: [
              { label: "Last hour", value: "hour" },
              { label: "Last 24 hours", value: "day" },
              { label: "Last 7 days", value: "week" },
            ],
          },
        ]}
        columns={[
          {
            header: "",
            className: "w-6 pr-0",
            cell: (e) => (
              <span
                className={cn(
                  "block h-2 w-2 rounded-full",
                  e.read_at ? "bg-transparent" : "bg-brand-sky",
                )}
                title={e.read_at ? "Read" : "Unread"}
              />
            ),
          },
          {
            header: "Event",
            cell: (e) => (
              <span className="flex flex-col">
                <span className={cn("text-sm", !e.read_at && "font-semibold")}>
                  {topics.describe(e.topic)}
                </span>
                <span className="font-mono text-xs text-ink/50">{e.topic}</span>
              </span>
            ),
          },
          {
            header: "Area",
            cell: (e) => (
              <Badge variant="outline" size="sm">
                {topics.area(e.topic)}
              </Badge>
            ),
          },
          {
            header: "Record",
            cell: (e) => <EventFeed.RecordLink event={e} topics={topics} />,
          },
          {
            header: "When",
            accessorKey: "timestamp",
            sortKey: "created_at",
            cell: (e) => <RelativeTime date={e.timestamp} className="text-sm text-ink/70" />,
          },
        ]}
      />

      <Sheet open={!!selected} onOpenChange={(o) => !o && setSelected(null)}>
        <SheetContent className="w-full overflow-y-auto sm:max-w-xl">
          {selected && (
            <>
              <SheetHeader className="mb-4">
                <SheetTitle>{topics.describe(selected.topic)}</SheetTitle>
                <SheetDescription className="font-mono text-xs">{selected.topic}</SheetDescription>
              </SheetHeader>
              <FieldGrid
                columns={2}
                className="mb-5"
                fields={[
                  { label: "Published", value: <RelativeTime date={selected.timestamp} /> },
                  { label: "Source", value: selected.source_crate },
                  {
                    label: "Record",
                    value: <EventFeed.RecordLink event={selected} topics={topics} />,
                  },
                  { label: "Visibility", value: selected.visibility },
                  { label: "Event", value: <Identifier value={selected.id} />, span: 2 },
                  {
                    label: "Correlation",
                    value: <Identifier value={selected.correlation_id} />,
                    span: 2,
                    hidden: !selected.correlation_id,
                  },
                ]}
              />
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-ink/60">
                Payload
              </h3>
              <JsonTable value={selected.payload as any} />
            </>
          )}
        </SheetContent>
      </Sheet>
    </>
  );
};

/** Link to the page of the record an event is about, if the app has one. */
EventFeed.RecordLink = ({ event, topics }: { event: EventEnvelope; topics: EventTopicCatalog }) => {
  const link = topics.link(event);
  if (!link) return <span className="text-ink/35">—</span>;
  return (
    <Link
      to={link.to as any}
      params={link.params as any}
      onClick={(e: React.MouseEvent) => e.stopPropagation()}
      className="inline-flex items-center gap-1 text-brand-sky hover:underline"
    >
      <Identifier value={link.label} copyable={false} />
      <ArrowUpRight className="h-3.5 w-3.5" />
    </Link>
  );
};
