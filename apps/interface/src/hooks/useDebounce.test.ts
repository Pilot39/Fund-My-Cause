import { renderHook, act } from "@testing-library/react";
import { useDebounce } from "./useDebounce";

beforeEach(() => jest.useFakeTimers());
afterEach(() => jest.useRealTimers());

describe("useDebounce", () => {
  it("returns the initial value immediately", () => {
    const { result } = renderHook(() => useDebounce("hello", 300));
    expect(result.current).toBe("hello");
  });

  it("does not update before the delay elapses", () => {
    const { result, rerender } = renderHook(({ v }) => useDebounce(v, 300), {
      initialProps: { v: "a" },
    });
    rerender({ v: "b" });
    act(() => jest.advanceTimersByTime(200));
    expect(result.current).toBe("a");
  });

  it("updates after the delay elapses", () => {
    const { result, rerender } = renderHook(({ v }) => useDebounce(v, 300), {
      initialProps: { v: "a" },
    });
    rerender({ v: "b" });
    act(() => jest.advanceTimersByTime(300));
    expect(result.current).toBe("b");
  });

  it("resets the timer on rapid changes (only last value wins)", () => {
    const { result, rerender } = renderHook(({ v }) => useDebounce(v, 300), {
      initialProps: { v: "a" },
    });
    rerender({ v: "b" });
    act(() => jest.advanceTimersByTime(100));
    rerender({ v: "c" });
    act(() => jest.advanceTimersByTime(100));
    rerender({ v: "d" });
    act(() => jest.advanceTimersByTime(300));
    expect(result.current).toBe("d");
  });

  it("uses 300 ms as the default delay", () => {
    const { result, rerender } = renderHook(({ v }) => useDebounce(v), {
      initialProps: { v: 1 },
    });
    rerender({ v: 2 });
    act(() => jest.advanceTimersByTime(299));
    expect(result.current).toBe(1);
    act(() => jest.advanceTimersByTime(1));
    expect(result.current).toBe(2);
  });

  it("works with object values", () => {
    const a = { x: 1 };
    const b = { x: 2 };
    const { result, rerender } = renderHook(({ v }) => useDebounce(v, 200), {
      initialProps: { v: a },
    });
    rerender({ v: b });
    act(() => jest.advanceTimersByTime(200));
    expect(result.current).toEqual({ x: 2 });
  });
});
