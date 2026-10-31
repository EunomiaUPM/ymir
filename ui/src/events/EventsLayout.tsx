import React, { ReactNode } from "react";
import { PageLayout } from "../layout/PageLayout";
import { PageHeader } from "../layout/PageHeader";
import { RouteTabs } from "../layout/RouteTabs";

export interface EventsLayoutProps {
  /** Route the three tabs hang from, e.g. `/events`. */
  basePath: string;
  description: string;
  /** The current tab, usually the router's `<Outlet />`. */
  children: ReactNode;
}

/** Events page with its Feed, Webhook subscriptions and Dead letters tabs. */
export const EventsLayout = ({ basePath, description, children }: EventsLayoutProps) => (
  <PageLayout>
    <PageHeader title="Events" description={description} />
    <RouteTabs
      tabs={[
        { label: "Feed", to: `${basePath}/feed` },
        { label: "Webhook subscriptions", to: `${basePath}/subscriptions` },
        { label: "Dead letters", to: `${basePath}/dlq` },
      ]}
    />
    {children}
  </PageLayout>
);
