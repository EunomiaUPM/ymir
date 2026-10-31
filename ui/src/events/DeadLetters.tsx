import React, { useState } from "react";
import { keepPreviousData } from "@tanstack/react-query";
import { AlertTriangle, RefreshCw, Trash2, Eye, RotateCcw } from "lucide-react";
import { toast } from "sonner";
import { useTableQueryParams } from "../hooks/useTableQueryParams";
import { PageSection } from "../layout/PageSection";
import { DataTable } from "../table/DataTable";
import { FormatDate } from "../ui/format-date";
import { Skeleton } from "../ui/skeleton";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "../ui/dialog";
import { EventsApi } from "./api";
import { EventsQueries } from "./queries";
import type { DeadLetterRecord, DeadLetterStatus } from "./types";

/** Deliveries that kept failing: inspect, replay one or all, or discard. */
export const DeadLetters = () => {
  const [selectedDlq, setSelectedDlq] = useState<DeadLetterRecord | null>(null);

  const { params: queryParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { status: "all" },
  });

  const selectedStatus =
    queryParams.filters.status && queryParams.filters.status !== "all"
      ? (queryParams.filters.status as DeadLetterStatus)
      : undefined;

  const { data, isLoading, isError, refetch, isFetching } = EventsQueries.useDeadLetters(
    {
      status: selectedStatus,
      limit: queryParams.limit,
      page: queryParams.cursor ? undefined : queryParams.page,
      cursor: queryParams.cursor,
      sort: queryParams.sort,
    },
    { placeholderData: keepPreviousData },
  );

  const deadLetters: DeadLetterRecord[] = Array.isArray(data?.data)
    ? (data.data as DeadLetterRecord[])
    : [];

  const { mutate: replaySingle, isPending: isReplayingSingle } = EventsQueries.useAction(
    EventsApi.replayDeadLetter,
    {
      onSuccess: () => toast.success("Event re-enqueued for delivery"),
      onError: (err) => toast.error(`Replay failed: ${String(err)}`),
    },
  );

  const { mutate: replayAll, isPending: isReplayingAll } = EventsQueries.useAction(
    EventsApi.replayAllDeadLetters,
    {
      onSuccess: (res) => toast.success(`Replayed ${res.data.replayed_count} dead letters`),
      onError: (err) => toast.error(`Replay all failed: ${String(err)}`),
    },
  );

  const { mutate: deleteRecord } = EventsQueries.useAction(EventsApi.deleteDeadLetter, {
    onSuccess: () => toast.success("Dead letter record discarded"),
    onError: (err) => toast.error(`Discard failed: ${String(err)}`),
  });

  return (
    <div className="flex flex-col gap-6 w-full">
      <div className="flex flex-col sm:flex-row sm:items-center justify-end gap-4">
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            disabled={isFetching}
            className="gap-1.5"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isFetching ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button
            size="sm"
            onClick={() => replayAll(undefined)}
            disabled={isReplayingAll || deadLetters.length === 0}
            className="gap-1.5"
          >
            <RotateCcw className="h-3.5 w-3.5" />
            {isReplayingAll ? "Replaying All..." : "Replay All"}
          </Button>
        </div>
      </div>

      {/* Filter Chips */}
      <div className="flex items-center gap-2">
        {[
          { label: "Unresolved", value: "Unresolved" },
          { label: "Replayed", value: "Replayed" },
          { label: "Purged", value: "Purged" },
        ].map((item) => (
          <Button
            key={item.value}
            variant={(queryParams.filters.status ?? "all") === item.value ? "default" : "outline"}
            size="sm"
            className="text-xs h-7"
            onClick={() => onQueryChange({ filters: { status: item.value }, page: 1 })}
          >
            {item.label}
          </Button>
        ))}
      </div>

      <PageSection>
        {isLoading && !data ? (
          <div className="flex flex-col gap-3 p-4">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : isError && !data ? (
          <div className="p-8 text-center text-sm text-destructive">
            Failed to load Dead Letter Queue records.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={deadLetters}
            serverSide={true}
            loading={isFetching}
            queryParams={queryParams}
            onQueryChange={onQueryChange}
            keyExtractor={(dlq: DeadLetterRecord) => dlq.id}
            searchPlaceholder="Filter dead letters by topic, status, or callback..."
            emptyMessage="No dead letter messages found. All deliveries healthy."
            defaultSortKey="created_at"
            defaultSortDirection="desc"
            pageSize={10}
            pageSizeOptions={[10, 20, 50, 100]}
            filters={[
              {
                id: "status",
                label: "Status",
                value: queryParams.filters.status ?? "all",
                options: [
                  { label: "Unresolved", value: "Unresolved" },
                  { label: "Replayed", value: "Replayed" },
                  { label: "Purged", value: "Purged" },
                ],
              },
            ]}
            columns={[
              {
                header: "Topic",
                accessorKey: "topic",
                sortable: false,
                cell: (dlq: DeadLetterRecord) => <Badge variant="code">{dlq.topic}</Badge>,
              },
              {
                header: "Status",
                accessorKey: "status",
                sortable: false,
                cell: (dlq: DeadLetterRecord) => (
                  <Badge variant={dlq.status === "Unresolved" ? "destructive" : "default"}>
                    {dlq.status}
                  </Badge>
                ),
              },
              {
                header: "Error",
                accessorKey: "error_message",
                sortable: false,
                cell: (dlq: DeadLetterRecord) => (
                  <span className="text-danger-700 dark:text-danger-300 text-xs block max-w-[280px] truncate">
                    {dlq.error_message}
                  </span>
                ),
              },
              {
                header: "Callback",
                accessorKey: "callback_address",
                sortable: false,
                cell: (dlq: DeadLetterRecord) => (
                  <span className="font-mono text-xs text-muted-foreground block max-w-[240px] truncate">
                    {dlq.callback_address}
                  </span>
                ),
              },
              {
                header: "Attempts",
                accessorKey: "attempts",
                sortable: false,
                cell: (dlq: DeadLetterRecord) => <Badge variant="infoLighter">{dlq.attempts}</Badge>,
              },
              {
                header: "Failed at",
                sortKey: "created_at",
                cell: (dlq: DeadLetterRecord) => <FormatDate date={dlq.failed_at} />,
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (dlq: DeadLetterRecord) => (
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="xs"
                      className="gap-1"
                      disabled={isReplayingSingle}
                      onClick={() => replaySingle(dlq.id)}
                    >
                      <RotateCcw className="h-3 w-3" /> Replay
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7"
                      onClick={() => setSelectedDlq(dlq)}
                    >
                      <Eye className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                      onClick={() => {
                        if (confirm("Discard this dead letter record?")) {
                          deleteRecord(dlq.id);
                        }
                      }}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                ),
              },
            ]}
          />
        )}
      </PageSection>

      {/* Detail Dialog */}
      <Dialog open={!!selectedDlq} onOpenChange={() => setSelectedDlq(null)}>
        <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-sm font-mono text-destructive">
              <AlertTriangle className="h-4 w-4" />
              Dead Letter: {selectedDlq?.id}
            </DialogTitle>
            <DialogDescription className="text-xs text-muted-foreground">
              Dead letter queue inspection for failed event delivery.
            </DialogDescription>
          </DialogHeader>
          <div className="flex-1 overflow-y-auto rounded bg-muted/40 p-4 font-mono text-xs flex flex-col gap-3">
            <div>
              <span className="text-muted-foreground font-semibold">Error Message:</span>
              <p className="text-destructive mt-1 font-sans">{selectedDlq?.error_message}</p>
            </div>
            <div>
              <span className="text-muted-foreground font-semibold">Target Callback:</span>
              <p className="text-foreground mt-1 select-all">{selectedDlq?.callback_address}</p>
            </div>
            <div>
              <span className="text-muted-foreground font-semibold">Event Payload:</span>
              <pre className="whitespace-pre-wrap break-all mt-1 bg-background/60 p-2.5 rounded border border-ink/5">
                {JSON.stringify(selectedDlq?.payload, null, 2)}
              </pre>
            </div>
          </div>
          <DialogFooter>
            <Button onClick={() => setSelectedDlq(null)}>Close</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
