import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useCrudTab, type CrudOperations } from "../hooks/useCrudTab";

function createMockOperations(): CrudOperations<{ id: string; name: string }> {
  return {
    getAll: vi.fn().mockResolvedValue([]),
    save: vi.fn().mockResolvedValue(undefined),
    delete: vi.fn().mockResolvedValue(undefined),
    toggle: vi.fn().mockResolvedValue(undefined),
  };
}

describe("useCrudTab error handling", () => {
  let operations: CrudOperations<{ id: string; name: string }>;

  beforeEach(() => {
    operations = createMockOperations();
  });

  it("should update items when getAll succeeds", async () => {
    const mockItems = [
      { id: "1", name: "Item 1" },
      { id: "2", name: "Item 2" },
    ];
    operations.getAll = vi.fn().mockResolvedValue(mockItems);

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual(mockItems);
    });
    expect(result.current.error).toBeNull();
  });

  it("should set error when getAll fails", async () => {
    const errorMessage = "Failed to fetch items";
    operations.getAll = vi.fn().mockRejectedValue(new Error(errorMessage));

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.error).toBe(`Error: ${errorMessage}`);
    });
  });

  it("should clear error when save succeeds", async () => {
    operations.getAll = vi
      .fn()
      .mockRejectedValueOnce(new Error("Initial error"))
      .mockResolvedValue([{ id: "1", name: "Item 1" }]);

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.error).toBeTruthy();
    });

    operations.save = vi.fn().mockResolvedValue(undefined);

    await act(async () => {
      await result.current.handleSave({ id: "1", name: "Item 1" });
    });

    expect(result.current.error).toBeNull();
  });

  it("should set error when save fails", async () => {
    const errorMessage = "Failed to save item";
    operations.save = vi.fn().mockRejectedValue(new Error(errorMessage));

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual([]);
    });

    await act(async () => {
      await result.current.handleSave({ id: "1", name: "New Item" });
    });

    expect(result.current.error).toBe(`Error: ${errorMessage}`);
  });

  it("should set error when delete fails", async () => {
    const errorMessage = "Failed to delete item";
    operations.delete = vi.fn().mockRejectedValue(new Error(errorMessage));

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual([]);
    });

    await act(async () => {
      await result.current.handleDelete("1");
    });

    expect(result.current.error).toBe(`Error: ${errorMessage}`);
  });

  it("should set error when toggle fails", async () => {
    const errorMessage = "Failed to toggle item";
    operations.toggle = vi.fn().mockRejectedValue(new Error(errorMessage));
    operations.getAll = vi
      .fn()
      .mockResolvedValueOnce([])
      .mockRejectedValue(new Error("Reload also failed"));

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual([]);
    });

    await act(async () => {
      await result.current.handleToggle("1", true);
    });

    expect(result.current.error).toBe("Error: Reload also failed");
  });

  it("should clear error to null when clearError is called", async () => {
    operations.getAll = vi.fn().mockRejectedValue(new Error("Some error"));

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.error).toBeTruthy();
    });

    act(() => {
      result.current.clearError();
    });

    expect(result.current.error).toBeNull();
  });

  it("should set isLoading to true during save operation", async () => {
    operations.save = vi.fn().mockImplementation(
      () =>
        new Promise((resolve) => {
          setTimeout(resolve, 100);
        }),
    );

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual([]);
    });

    act(() => {
      void result.current.handleSave({ id: "1", name: "New Item" });
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
      },
      { timeout: 200 },
    );
  });

  it("should set isLoading to true during delete operation", async () => {
    operations.delete = vi.fn().mockImplementation(
      () =>
        new Promise((resolve) => {
          setTimeout(resolve, 100);
        }),
    );

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual([]);
    });

    act(() => {
      void result.current.handleDelete("1");
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
      },
      { timeout: 200 },
    );
  });

  it("should set isLoading to true during toggle operation", async () => {
    operations.toggle = vi.fn().mockImplementation(
      () =>
        new Promise((resolve) => {
          setTimeout(resolve, 100);
        }),
    );

    const { result } = renderHook(() => useCrudTab(operations));

    await waitFor(() => {
      expect(result.current.items).toEqual([]);
    });

    act(() => {
      void result.current.handleToggle("1", true);
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(true);
    });

    await waitFor(
      () => {
        expect(result.current.isLoading).toBe(false);
      },
      { timeout: 200 },
    );
  });
});
