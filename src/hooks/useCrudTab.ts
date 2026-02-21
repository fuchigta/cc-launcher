import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CrudCommands {
  get: string;
  save: string;
  delete: string;
  toggle: string;
}

interface UseCrudTabResult<T> {
  items: T[];
  showForm: boolean;
  editingItem: T | null;
  loadItems: () => Promise<void>;
  handleSave: (paramName: string, item: T) => Promise<void>;
  handleDelete: (id: string) => Promise<void>;
  handleToggle: (id: string, enabled: boolean) => Promise<void>;
  handleEdit: (item: T) => void;
  handleNew: () => void;
  closeForm: () => void;
}

export function useCrudTab<T>(
  commands: CrudCommands,
  afterMutate?: () => Promise<void>,
): UseCrudTabResult<T> {
  const [items, setItems] = useState<T[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [editingItem, setEditingItem] = useState<T | null>(null);

  const loadItems = useCallback(async () => {
    try {
      const data = await invoke<T[]>(commands.get);
      setItems(data);
    } catch (e) {
      console.error(`Failed to load (${commands.get}):`, e);
    }
  }, [commands.get]);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  const handleSave = async (paramName: string, item: T) => {
    try {
      await invoke(commands.save, { [paramName]: item });
      setShowForm(false);
      setEditingItem(null);
      await loadItems();
      if (afterMutate) await afterMutate();
    } catch (e) {
      console.error(`Failed to save (${commands.save}):`, e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke(commands.delete, { id });
      await loadItems();
      if (afterMutate) await afterMutate();
    } catch (e) {
      console.error(`Failed to delete (${commands.delete}):`, e);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await invoke(commands.toggle, { id, enabled });
    } catch (e) {
      console.error(`Failed to toggle (${commands.toggle}):`, e);
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
