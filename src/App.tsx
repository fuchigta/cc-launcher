import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppConfig } from "./types";

function App() {
  const [prompt, setPrompt] = useState("");
  const [currentDirectory, setCurrentDirectory] = useState<string | null>(null);
  const [recentDirectories, setRecentDirectories] = useState<string[]>([]);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const isDialogOpenRef = useRef(false);

  useEffect(() => {
    const loadConfig = async () => {
      const config = await invoke<AppConfig>("get_config");
      setCurrentDirectory(config.lastDirectory);
      setRecentDirectories(config.recentDirectories);
    };
    loadConfig();

    const currentWindow = getCurrentWindow();

    const unlistenFocus = currentWindow.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        loadConfig();
        if (inputRef.current) {
          inputRef.current.focus();
          inputRef.current.select();
        }
        setDropdownOpen(false);
      }
    });

    if (inputRef.current) {
      inputRef.current.focus();
    }

    return () => {
      unlistenFocus.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    };

    if (dropdownOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [dropdownOpen]);

  // Resize window when dropdown opens/closes
  useEffect(() => {
    const currentWindow = getCurrentWindow();
    const baseHeight = 120;
    const bottomPadding = 12;
    // Calculate dropdown height: items (40px each) + browse (40px) + divider (9px) + margin (4px)
    const dropdownHeight =
      recentDirectories.length * 40 + 40 + (recentDirectories.length > 0 ? 9 : 0) + 4;
    const expandedHeight = baseHeight + dropdownHeight + bottomPadding;

    if (dropdownOpen) {
      currentWindow.setSize(new LogicalSize(600, expandedHeight));
    } else {
      currentWindow.setSize(new LogicalSize(600, baseHeight));
    }
  }, [dropdownOpen, recentDirectories.length]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (prompt.trim()) {
      try {
        if (currentDirectory) {
          await invoke("update_recent_directory", { directory: currentDirectory });
        }
        await invoke("open_claude_interactive", {
          prompt: prompt.trim(),
          workingDir: currentDirectory,
        });
        setPrompt("");
        await invoke("hide_window");
      } catch (error) {
        console.error("Failed to launch Claude:", error);
      }
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      if (dropdownOpen) {
        setDropdownOpen(false);
      } else {
        setPrompt("");
        await invoke("hide_window");
      }
    }
  };

  const handleBlur = async (e: React.FocusEvent) => {
    // Check if focus moved to another element within our container
    const relatedTarget = e.relatedTarget as Node | null;
    if (containerRef.current?.contains(relatedTarget)) {
      return;
    }

    setTimeout(async () => {
      // Don't hide if dialog is open
      if (isDialogOpenRef.current) {
        return;
      }

      const currentWindow = getCurrentWindow();
      const isFocused = await currentWindow.isFocused();
      if (!isFocused) {
        setPrompt("");
        setDropdownOpen(false);
        await invoke("hide_window");
      }
    }, 100);
  };

  const handleDirectoryClick = () => {
    setDropdownOpen(!dropdownOpen);
  };

  const handleSelectDirectory = (dir: string) => {
    setCurrentDirectory(dir);
    setDropdownOpen(false);
    inputRef.current?.focus();
  };

  const handleBrowse = async () => {
    isDialogOpenRef.current = true;
    setDropdownOpen(false);

    const currentWindow = getCurrentWindow();

    try {
      // Disable alwaysOnTop so dialog appears in front
      await currentWindow.setAlwaysOnTop(false);

      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: currentDirectory ?? undefined,
      });

      if (selected && typeof selected === "string") {
        setCurrentDirectory(selected);
        setRecentDirectories((prev) => {
          const filtered = prev.filter((d) => d !== selected);
          return [selected, ...filtered].slice(0, 5);
        });
        await invoke("update_recent_directory", { directory: selected });
      }
    } finally {
      // Restore alwaysOnTop
      await currentWindow.setAlwaysOnTop(true);
      isDialogOpenRef.current = false;
      inputRef.current?.focus();
    }
  };

  const displayDirectory = currentDirectory ?? "(No directory selected)";

  return (
    <div className="overlay-container" ref={containerRef}>
      <form onSubmit={handleSubmit} className="input-form" onBlur={handleBlur}>
        <input
          ref={inputRef}
          type="text"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask Claude..."
          className="prompt-input"
          autoFocus
        />
        <div className="directory-row" ref={dropdownRef}>
          <button type="button" className="directory-button" onClick={handleDirectoryClick}>
            <span className="directory-icon">&#128193;</span>
            <span className="directory-path">{displayDirectory}</span>
            <span className="dropdown-arrow">{dropdownOpen ? "\u25B2" : "\u25BC"}</span>
          </button>
          {dropdownOpen && (
            <div className="directory-dropdown">
              {recentDirectories.map((dir) => (
                <button
                  type="button"
                  key={dir}
                  className={`dropdown-item ${dir === currentDirectory ? "active" : ""}`}
                  onClick={() => handleSelectDirectory(dir)}
                >
                  {dir === currentDirectory && <span className="check-mark">&#9679;</span>}
                  <span className="dropdown-item-path">{dir}</span>
                </button>
              ))}
              {recentDirectories.length > 0 && <div className="dropdown-divider" />}
              <button type="button" className="dropdown-item browse-item" onClick={handleBrowse}>
                <span className="browse-icon">&#128194;</span>
                <span>Browse...</span>
              </button>
            </div>
          )}
        </div>
      </form>
    </div>
  );
}

export default App;
