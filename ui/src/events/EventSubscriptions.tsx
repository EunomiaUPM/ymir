import React, { useState } from "react";
import { keepPreviousData } from "@tanstack/react-query";
import { Plus, Trash2, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { useTableQueryParams } from "../hooks/useTableQueryParams";
import { PageSection } from "../layout/PageSection";
import { DataTable } from "../table/DataTable";
import { FormatDate } from "../ui/format-date";
import { Skeleton } from "../ui/skeleton";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";
import { Input } from "../ui/input";
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
import type { EventSubscription, EventTopicCatalog } from "./types";

interface CreateSubscriptionDialogProps {
  open: boolean;
  onClose: () => void;
  topics: EventTopicCatalog;
  defaultPattern: string;
}

const CreateSubscriptionDialog = ({
  open,
  onClose,
  topics,
  defaultPattern,
}: CreateSubscriptionDialogProps) => {
  const [topicPattern, setTopicPattern] = useState(defaultPattern);
  const [callbackAddress, setCallbackAddress] = useState("https://");
  const [secret, setSecret] = useState("");
  const [retryLimit, setRetryLimit] = useState("5");

  const topicPresets = [
    ...topics.areas.map((a) => ({ label: a.pattern, pattern: a.pattern })),
    { label: "* (all)", pattern: "*" },
  ];

  const { mutate: createSub, isPending } = EventsQueries.useAction(EventsApi.createSubscription, {
    onSuccess: () => {
      toast.success("Webhook subscription created");
      onClose();
      setSecret("");
    },
    onError: (err) => toast.error(`Subscription creation failed: ${String(err)}`),
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
      secret: secret.trim() || undefined,
      retry_limit: isNaN(retries) ? undefined : retries,
    });
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Add Webhook Subscription</DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            Configure an external webhook listener with automated retry policy and HMAC signing.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Topic Pattern
            </label>
            <Input
              value={topicPattern}
              onChange={(e) => setTopicPattern(e.target.value)}
              placeholder={`e.g. ${defaultPattern}`}
            />
            <div className="flex flex-wrap items-center gap-1.5 mt-1">
              <span className="text-[11px] text-muted-foreground">Presets:</span>
              {topicPresets.map((p) => (
                <Button
                  key={p.pattern}
                  type="button"
                  variant={topicPattern === p.pattern ? "default" : "outline"}
                  size="xs"
                  className="text-[11px] h-5 px-2 font-mono"
                  onClick={() => setTopicPattern(p.pattern)}
                >
                  {p.label}
                </Button>
              ))}
            </div>
            <p className="text-[11px] text-muted-foreground mt-0.5">
              Supports <code className="font-mono">:</code> and <code className="font-mono">.</code> delimiters with <code className="font-mono">*</code> (single segment) and <code className="font-mono">**</code> (multi-segment) wildcards.
            </p>
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Callback URL
            </label>
            <Input
              value={callbackAddress}
              onChange={(e) => setCallbackAddress(e.target.value)}
              placeholder="https://your-service.com/webhook"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              HMAC-SHA256 Secret (Optional)
            </label>
            <Input
              type="password"
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              placeholder="Leave empty for unauthenticated webhook"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Max Retry Limit
            </label>
            <Input
              type="number"
              value={retryLimit}
              onChange={(e) => setRetryLimit(e.target.value)}
              min="1"
              max="20"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isPending}>
            {isPending ? "Subscribing..." : "Create Subscription"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export interface EventSubscriptionsProps {
  topics: EventTopicCatalog;
  /** Pattern the new-subscription dialog starts with, e.g. `heimdall:**`. */
  defaultPattern: string;
}

/** Webhook subscriptions of the bus: list, create with HMAC secret and retries, delete. */
export const EventSubscriptions = ({ topics, defaultPattern }: EventSubscriptionsProps) => {
  const [createOpen, setCreateOpen] = useState(false);

  const { params: queryParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { status: "all" },
  });

  const status = queryParams.filters.status;
  const { data, isLoading, isError, isFetching } = EventsQueries.useSubscriptions(
    {
      active: status === "active" ? true : status === "inactive" ? false : undefined,
      limit: queryParams.limit,
      page: queryParams.cursor ? undefined : queryParams.page,
      cursor: queryParams.cursor,
      sort: queryParams.sort,
    },
    { placeholderData: keepPreviousData },
  );
  const subscriptions: EventSubscription[] = Array.isArray(data?.data)
    ? (data.data as EventSubscription[])
    : [];

  const { mutate: deleteSub } = EventsQueries.useAction(EventsApi.deleteSubscription, {
    onSuccess: () => toast.success("Subscription deleted"),
    onError: (err) => toast.error(`Delete failed: ${String(err)}`),
  });

  return (
    <div className="flex flex-col gap-6 w-full">
      <PageSection
        title="Webhooks"
        action={
          <Button size="sm" onClick={() => setCreateOpen(true)}>
            <Plus /> New subscription
          </Button>
        }
      >
        {isLoading && !data ? (
          <div className="flex flex-col gap-3 p-4">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : isError && !data ? (
          <div className="p-8 text-center text-sm text-destructive">
            Failed to load event subscriptions.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={subscriptions}
            serverSide={true}
            loading={isFetching}
            queryParams={queryParams}
            onQueryChange={onQueryChange}
            keyExtractor={(sub: EventSubscription) => sub.id}
            searchPlaceholder="Filter subscriptions by topic or callback URL..."
            emptyMessage="No webhook subscriptions configured yet."
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
                  { label: "Active", value: "active" },
                  { label: "Inactive", value: "inactive" },
                ],
              },
            ]}
            columns={[
              {
                header: "Topic pattern",
                accessorKey: "topic_pattern",
                sortable: false,
                cell: (sub: EventSubscription) => <Badge variant="code">{sub.topic_pattern}</Badge>,
              },
              {
                header: "Status",
                accessorKey: "active",
                sortable: false,
                cell: (sub: EventSubscription) => (
                  <Badge variant="status" state={sub.active ? "ACTIVE" : "PAUSE"}>
                    {sub.active ? "Active" : "Inactive"}
                  </Badge>
                ),
              },
              {
                header: "Callback",
                accessorKey: "callback_address",
                sortable: false,
                cell: (sub: EventSubscription) => (
                  <span className="font-mono text-xs block max-w-[280px] truncate">
                    {sub.callback_address}
                  </span>
                ),
              },
              {
                header: "Signature",
                sortable: false,
                searchable: false,
                cell: (sub: EventSubscription) =>
                  sub.secret ? (
                    <Badge variant="success">
                      <ShieldCheck className="h-3 w-3" /> HMAC
                    </Badge>
                  ) : (
                    <span className="text-xs text-muted-foreground">None</span>
                  ),
              },
              {
                header: "Retries",
                accessorKey: "retry_limit",
                sortable: false,
                cell: (sub: EventSubscription) => <Badge variant="infoLighter">{sub.retry_limit ?? 5}</Badge>,
              },
              {
                header: "Created at",
                accessorKey: "created_at",
                cell: (sub: EventSubscription) => <FormatDate date={sub.created_at} />,
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (sub: EventSubscription) => (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                    onClick={() => {
                      if (confirm(`Delete subscription for "${sub.topic_pattern}"?`)) {
                        deleteSub(sub.id);
                      }
                    }}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                ),
              },
            ]}
          />
        )}
      </PageSection>

      <CreateSubscriptionDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        topics={topics}
        defaultPattern={defaultPattern}
      />
    </div>
  );
};
