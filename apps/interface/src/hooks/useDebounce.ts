import { useState, useEffect } from "react";

/**
 * Returns a debounced copy of `value` that only updates after `delay` ms of inactivity.
 * Useful for search inputs, API calls, and expensive re-renders.
 *
 * @param value - value to debounce
 * @param delay - debounce delay in milliseconds (default: 300)
 * @returns debounced value
 *
 * @example
 * const debouncedSearch = useDebounce(searchTerm, 400);
 * useEffect(() => { fetchResults(debouncedSearch); }, [debouncedSearch]);
 */
export function useDebounce<T>(value: T, delay = 300): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}
