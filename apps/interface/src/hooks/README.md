# Custom React Hooks

Reusable utility hooks for the Fund-My-Cause frontend. All hooks are fully typed and tree-shakeable via `src/hooks/index.ts`.

---

## useLocalStorage

Persists state to `localStorage` with JSON serialization. Falls back gracefully when `localStorage` is unavailable (SSR, private browsing).

```ts
const [value, setValue, removeValue] = useLocalStorage<T>(key: string, initialValue: T)
```

| Return | Type | Description |
|--------|------|-------------|
| `value` | `T` | Current stored value |
| `setValue` | `(value: T \| ((prev: T) => T)) => void` | Update and persist the value |
| `removeValue` | `() => void` | Delete from storage and reset to `initialValue` |

**Example**

```tsx
const [theme, setTheme, removeTheme] = useLocalStorage("theme", "light");

// Toggle theme
setTheme((prev) => (prev === "light" ? "dark" : "light"));

// Clear preference
removeTheme();
```

---

## useDebounce

Returns a debounced copy of `value` that only updates after `delay` ms of inactivity. Cancels the pending update if the component unmounts.

```ts
const debouncedValue = useDebounce<T>(value: T, delay?: number): T
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `value` | `T` | — | Value to debounce |
| `delay` | `number` | `300` | Debounce delay in milliseconds |

**Example**

```tsx
const [query, setQuery] = useState("");
const debouncedQuery = useDebounce(query, 400);

useEffect(() => {
  if (debouncedQuery) fetchResults(debouncedQuery);
}, [debouncedQuery]);
```

---

## useAsync

Manages the full lifecycle of an async operation: `idle → loading → success | error`. Automatically ignores stale responses when a newer call is in flight.

```ts
const { data, error, status, isLoading, isSuccess, isError, execute, reset } =
  useAsync<T>(asyncFn: (...args: unknown[]) => Promise<T>)
```

| Return | Type | Description |
|--------|------|-------------|
| `data` | `T \| null` | Resolved value, or `null` |
| `error` | `string \| null` | Error message, or `null` |
| `status` | `"idle" \| "loading" \| "success" \| "error"` | Current state |
| `isLoading` | `boolean` | `true` while the promise is pending |
| `isSuccess` | `boolean` | `true` after a successful resolution |
| `isError` | `boolean` | `true` after a rejection |
| `execute` | `(...args) => Promise<T \| null>` | Trigger the async function |
| `reset` | `() => void` | Return to `idle` and clear data/error |

**Example**

```tsx
const { data: campaigns, isLoading, error, execute } = useAsync(fetchCampaigns);

useEffect(() => {
  execute();
}, [execute]);

if (isLoading) return <Spinner />;
if (error) return <ErrorMessage message={error} />;
```

---

## Domain Hooks

These hooks are specific to Fund-My-Cause and are also exported from `src/hooks/index.ts`.

| Hook | Description |
|------|-------------|
| `useCampaign(contractId)` | Fetch campaign info + stats, with optimistic contribution support |
| `useContribute(contractId)` | Mutation to pledge tokens |
| `useWithdraw(contractId)` | Mutation for creator to withdraw funds |
| `useRefund(contractId)` | Mutation for contributor to claim a refund |
| `useBatchRefund(contractId)` | Mutation to refund multiple contributors |
| `usePause(contractId)` | Mutation to pause a campaign |
| `useUnpause(contractId)` | Mutation to unpause a campaign |
| `useCampaignDraft(data)` | Auto-save campaign form data to localStorage |
| `useXlmBalance(address)` | Fetch native XLM balance via Horizon |
| `useAccountExists(address)` | Check whether a Stellar account is funded |
| `useRecommendations()` | Fetch recommended campaigns |
| `useComments()` | Manage campaign comments |

---

## Running Tests

```bash
cd apps/interface
npm test -- --testPathPattern=src/hooks
```
