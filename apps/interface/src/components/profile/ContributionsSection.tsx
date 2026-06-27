"use client";

import React from "react";
import { Loader2, RefreshCw } from "lucide-react";
import { useContributions } from "@/hooks/useContributions";

interface ContributionsSectionProps {
  address: string;
}

/**
 * Displays all contributions made by the given address, sorted by date descending.
 */
export function ContributionsSection({ address }: ContributionsSectionProps) {
  const { contributions, loading, error, retry } = useContributions(address);

  return (
    <section aria-labelledby="profile-contributions-heading" className="space-y-3">
      <h2 id="profile-contributions-heading" className="text-lg font-semibold">
        Contribution History
      </h2>

      {loading ? (
        <div className="flex justify-center py-8">
          <Loader2 size={24} className="animate-spin text-gray-400" />
        </div>
      ) : error ? (
        <div className="flex items-center gap-3 rounded-xl border border-red-800 bg-red-950/30 p-4">
          <p className="text-sm text-red-300 flex-1">{error}</p>
          <button
            onClick={retry}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg bg-red-800 hover:bg-red-700 text-white transition"
          >
            <RefreshCw size={12} />
            Retry
          </button>
        </div>
      ) : contributions.length === 0 ? (
        <p className="text-sm text-gray-500">No contributions made yet.</p>
      ) : (
        <div className="space-y-2">
          {contributions.map((entry) => (
            <div
              key={`${entry.contractId}-${entry.date}`}
              className="flex items-center justify-between gap-4 rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 px-4 py-3"
            >
              <div className="min-w-0">
                <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                  {entry.campaignTitle}
                </p>
                <p className="text-xs text-gray-500">
                  {new Date(entry.date).toLocaleDateString()}
                </p>
              </div>
              <span className="text-sm font-semibold text-indigo-500 shrink-0">
                {entry.amount.toLocaleString(undefined, { maximumFractionDigits: 2 })} XLM
              </span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
