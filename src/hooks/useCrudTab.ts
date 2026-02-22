import { useState, useEffect, useCallback } from "react";

export interface CrudOperations<T> {
  getAll: () => Promise<T[]>;
  save: (item: T) => Promise<unknown>;
  delete: (id: string) => Promise<unknown>;
  toggle: (id: string, enabled: boolean) => Promise<unknown>;
}

interface UseCrudTabResult<T> {
  items: T[];
  showForm: boolean;
  editingItem: T | null;
  error: string | null;
  isLoading: boolean;
  loadItems: () => Promise<void>;
  handleSave: (item: T) => Promise<void>;
  handleDelete: (id: string) => Promise<void>;
  handleToggle: (id: string, enabled: boolean) => Promise<void>;
  handleEdit: (item: T) => void;
  handleNew: () => void;
  closeForm: () => void;
  clearError: () => void;
}

export function useCrudTab<T>(
  operations: CrudOperations<T>,
  afterMutate?: () => Promise<void>,
): UseCrudTabResult<T> {
  const [items, setItems] = useState<T[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingItem, setEditingItem] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const loadItems = useCallback(async () => {
    try {
      const data = await operations.getAll();
      setItems(data);
      setError(null);
    } catch (e) {
      console.error("Failed to load items:", e);
      setError(String(e));
    }
  }, [operations.getAll]);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  const handleSave = async (item: T) => {
    setIsLoading(true);
    try {
      await operations.save(item);
      setShowForm(false);
      setEditingItem(null);
      await loadItems();
      if (afterMutate) await afterMutate();
      setError(null);
    } catch (e) {
      console.error("Failed to save item:", e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    setIsLoading(true);
    try {
      await operations.delete(id);
      await loadItems();
      if (afterMutate) await afterMutate();
      setError(null);
    } catch (e) {
      console.error("Failed to delete item:", e);
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    setIsLoading(true);
    try {
      await operations.toggle(id, enabled);
      await loadItems();
      if (afterMutate) await afterMutate();
      setError(null);
    } catch (e) {
      console.error("Failed to toggle item:", e);
      setError(String(e));
      await loadItems();
      if (afterMutate) await afterMutate();
    } finally {
      setIsLoading(false);
    }
  };

  const handleEdit = (item: T) => {
    setEditingItem(item);
    setShowForm(true);
  };

  const handleNew = () => {
    setEditingItem(null);
    setShowForm(true);
  };

  const closeForm = () => {
    setShowForm(false);
    setEditingItem(null);
  };

  const clearError = () => {
    setError(null);
  };

  return {
    items,
    showForm,
    editingItem,
    error,
    isLoading,
    loadItems,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
    clearError,
  };
}
