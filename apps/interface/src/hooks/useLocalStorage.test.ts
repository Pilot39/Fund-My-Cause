import { renderHook, act } from "@testing-library/react";
import { useLocalStorage } from "./useLocalStorage";

const KEY = "test-key";

beforeEach(() => localStorage.clear());

describe("useLocalStorage", () => {
  it("returns initialValue when nothing is stored", () => {
    const { result } = renderHook(() => useLocalStorage(KEY, "default"));
    expect(result.current[0]).toBe("default");
  });

  it("reads an existing value from localStorage", () => {
    localStorage.setItem(KEY, JSON.stringify("stored"));
    const { result } = renderHook(() => useLocalStorage(KEY, "default"));
    expect(result.current[0]).toBe("stored");
  });

  it("persists a new value to localStorage", () => {
    const { result } = renderHook(() => useLocalStorage(KEY, "default"));
    act(() => result.current[1]("updated"));
    expect(result.current[0]).toBe("updated");
    expect(JSON.parse(localStorage.getItem(KEY)!)).toBe("updated");
  });

  it("supports functional updater", () => {
    const { result } = renderHook(() => useLocalStorage(KEY, 0));
    act(() => result.current[1]((n) => n + 1));
    expect(result.current[0]).toBe(1);
  });

  it("removes the value and resets to initialValue", () => {
    const { result } = renderHook(() => useLocalStorage(KEY, "default"));
    act(() => result.current[1]("something"));
    act(() => result.current[2]());
    expect(result.current[0]).toBe("default");
    expect(localStorage.getItem(KEY)).toBeNull();
  });

  it("handles objects", () => {
    const { result } = renderHook(() => useLocalStorage<{ x: number }>(KEY, { x: 0 }));
    act(() => result.current[1]({ x: 42 }));
    expect(result.current[0]).toEqual({ x: 42 });
  });

  it("falls back to initialValue when stored JSON is corrupt", () => {
    localStorage.setItem(KEY, "not-json{{{");
    const { result } = renderHook(() => useLocalStorage(KEY, "fallback"));
    expect(result.current[0]).toBe("fallback");
  });
});
