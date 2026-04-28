import { renderHook, act, waitFor } from "@testing-library/react";
import { useAsync } from "./useAsync";

describe("useAsync", () => {
  it("starts in idle state", () => {
    const { result } = renderHook(() => useAsync(jest.fn()));
    expect(result.current.status).toBe("idle");
    expect(result.current.data).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.isLoading).toBe(false);
  });

  it("transitions to loading then success", async () => {
    const fn = jest.fn().mockResolvedValue("ok");
    const { result } = renderHook(() => useAsync(fn));

    act(() => { result.current.execute(); });
    expect(result.current.isLoading).toBe(true);

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.status).toBe("success");
    expect(result.current.data).toBe("ok");
    expect(result.current.isSuccess).toBe(true);
    expect(result.current.error).toBeNull();
  });

  it("transitions to error on rejection", async () => {
    const fn = jest.fn().mockRejectedValue(new Error("boom"));
    const { result } = renderHook(() => useAsync(fn));

    await act(() => result.current.execute());

    expect(result.current.status).toBe("error");
    expect(result.current.error).toBe("boom");
    expect(result.current.isError).toBe(true);
    expect(result.current.data).toBeNull();
  });

  it("converts non-Error rejections to string", async () => {
    const fn = jest.fn().mockRejectedValue("plain string error");
    const { result } = renderHook(() => useAsync(fn));

    await act(() => result.current.execute());
    expect(result.current.error).toBe("plain string error");
  });

  it("reset returns to idle and clears data", async () => {
    const fn = jest.fn().mockResolvedValue(42);
    const { result } = renderHook(() => useAsync(fn));

    await act(() => result.current.execute());
    expect(result.current.data).toBe(42);

    act(() => result.current.reset());
    expect(result.current.status).toBe("idle");
    expect(result.current.data).toBeNull();
  });

  it("ignores stale response when reset is called before resolution", async () => {
    let resolve!: (v: string) => void;
    const fn = jest.fn().mockReturnValue(new Promise<string>((r) => { resolve = r; }));
    const { result } = renderHook(() => useAsync(fn));

    act(() => { result.current.execute(); });
    act(() => { result.current.reset(); });
    act(() => { resolve("stale"); });

    // Should remain idle, not success
    await waitFor(() => expect(result.current.status).toBe("idle"));
    expect(result.current.data).toBeNull();
  });

  it("passes arguments through to the async function", async () => {
    const fn = jest.fn().mockResolvedValue("result");
    const { result } = renderHook(() => useAsync(fn));

    await act(() => result.current.execute("arg1", 2));
    expect(fn).toHaveBeenCalledWith("arg1", 2);
  });
});
