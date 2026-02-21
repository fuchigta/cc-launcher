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
  loadItems: () => Promise<void>;
  handleSave: (item: T) => Promise<void>;
  handleDelete: (id: string) => Promise<void>;
  handleToggle: (id: string, enabled: boolean) => Promise<void>;
  handleEdit: (item: T) => void;
  handleNew: () => void;
  closeForm: () => void;
}

export function useCrudTab<T>(
  operations: CrudOperations<T>,
  afterMutate?: () => Promise<void>,
): UseCrudTabResult<T> {
  const [items, setItems] = useState<T[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingItem, setEditingItem] = useState<T | null>(null);

  const loadItems = useCallback(async () => {
    try {
      const data = await operations.getAll();
      setItems(data);
    } catch (e) {
      console.error("Failed to load items:", e);
    }
  }, [operations.getAll]);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  const handleSave = async (item: T) => {
    try {
      await operations.save(item);
      setShowForm(false);
      setEditingItem(null);
      await loadItems();
      if (afterMutate) await afterMutate();
    } catch (e) {
      console.error("Failed to save item:", e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await operations.delete(id);
      await loadItems();
      if (afterMutate) await afterMutate();
    } catch (e) {
      console.error("Failed to delete item:", e);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await operations.toggle(id, enabled);
    } catch (e) {
      console.error("Failed to toggle item:", e);
    } finally {
      await loadItems();
      if (afterMutate) await afterMutate();
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

  return {
    items,
    showForm,
    editingItem,
    loadItems,
    handleSave,
    handleDelete,
    handleToggle,
    handleEdit,
    handleNew,
    closeForm,
  };
}
