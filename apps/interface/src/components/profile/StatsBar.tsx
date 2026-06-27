"use client";

import React from "react";

interface StatsBarProps {
  campaignCount: number;
  totalRaised: number;
  contributionCount: number;
  totalContributed: number;
  loading?: boolean;
}

function StatTile({
  label,
  value,
  loading,
}: {
  label: string;
  value: string;
  loading?: boolean;
}) {
  return (
    <div className="rounded-xl bg-gray-100 dark:bg-gray-900 border border-gray-200 dark:border-gray-800 p-4 text-center">
      {loading ? (
        <div className="h-6 w-16 mx-auto rounded bg-gray-300 dark:bg-gray-700 animate-pulse mb-1" />
      ) : (
        <p className="text-xl font-semibold text-gray-900 dark:text-white">{value}</p>
      )}
      <p className="text-xs text-gray-500 mt-1">{label}</p>
    </div>
  );
}

/**
 * Displays four aggregate stat tiles for a user profile.
 */
export function StatsBar({
  campaignCount,
  totalRaised,
  contributionCount,
  totalContributed,
  loading = false,
}: StatsBarProps) {
  const fmt = (n: number) =>
    n.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });

  return (
    <div
      className="grid grid-cols-2 sm:grid-cols-4 gap-4"
      aria-label="Profile statistics"
    >
      <StatTile
        label="Campaigns Created"
        value={String(campaignCount)}
        loading={loading}
      />
      <StatTile
        label="Total Raised (XLM)"
        value={`${fmt(totalRaised)} XLM`}
        loading={loading}
      />
      <StatTile
        label="Campaigns Backed"
        value={String(contributionCount)}
        loading={loading}
      />
      <StatTile
        label="Total Contributed (XLM)"
        value={`${fmt(totalContributed)} XLM`}
        loading={loading}
      />
    </div>
  );
}
