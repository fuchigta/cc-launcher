import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppConfig, TerminalType } from "./types";

function App() {
  const [prompt, setPrompt] = useState("");
  const [terminal, setTerminal] = useState<TerminalType>("Auto");
  const [currentDirectory, setCurrentDirectory] = useState<string | null>(null);
  const [recentDirectories, setRecentDirectories] = useState<string[]>([]);
  const [wslDirectory, setWslDirectory] = useState<string>("");
  const [wslRecentDirectories, setWslRecentDirectories] = useState<string[]>([]);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const formRef = useRef<HTMLFormElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const isDialogOpenRef = useRef(false);

  const isWsl = terminal === "Wsl";

  useEffect(() => {
    const loadConfig = async () => {
      const config = await invoke<AppConfig>("get_config");
      setTerminal(config.terminal);
      setCurrentDirectory(config.lastDirectory);
      setRecentDirectories(config.recentDirectories);
      setWslDirectory(config.wslDirectory ?? "");
      setWslRecentDirectories(config.wslRecentDirectories ?? []);
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
    const dirList = isWsl ? wslRecentDirectories : recentDirectories;
    const browseHeight = 40;
    const dropdownHeight = dirList.length * 40 + browseHeight + (dirList.length > 0 ? 9 : 0) + 4;
    const expandedHeight = baseHeight + dropdownHeight + bottomPadding;

    if (dropdownOpen) {
      currentWindow.setSize(new LogicalSize(600, expandedHeight));
    } else {
      currentWindow.setSize(new LogicalSize(600, baseHeight));
    }
  }, [dropdownOpen, recentDirectories.length, wslRecentDirectories.length, isWsl]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (prompt.trim()) {
      try {
        if (isWsl) {
          if (wslDirectory.trim()) {
            await invoke("update_wsl_directory", { directory: wslDirectory.trim() });
          }
          await invoke("open_claude_interactive", {
            prompt: prompt.trim(),
            workingDir: null,
          });
        } else {
          if (currentDirectory) {
            await invoke("update_recent_directory", { directory: currentDirectory });
          }
          await invoke("open_claude_interactive", {
            prompt: prompt.trim(),
            workingDir: currentDirectory,
          });
        }
        setPrompt("");
        await invoke("hide_window");
      } catch (error) {
        console.error("Failed to launch Claude:", error);
      }
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.nativeEvent.isComposing) {
      e.preventDefault();
      formRef.current?.requestSubmit();
    } else if (e.key === "Escape") {
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
    if (isWsl) {
      setWslDirectory(dir);
    } else {
      setCurrentDirectory(dir);
    }
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

  const handleWslBrowse = async () => {
    isDialogOpenRef.current = true;
    setDropdownOpen(false);

    const currentWindow = getCurrentWindow();

    try {
      // Disable alwaysOnTop so dialog appears in front
      await currentWindow.setAlwaysOnTop(false);

      // Get WSL root path as default
      const wslRoot = await invoke<string>("get_wsl_root_path");

      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: wslRoot,
      });

      if (selected && typeof selected === "string") {
        // Convert UNC path to WSL path
        const wslPath = await invoke<string>("unc_to_wsl_path", { uncPath: selected });
        setWslDirectory(wslPath);
        setWslRecentDirectories((prev) => {
          const filtered = prev.filter((d) => d !== wslPath);
          return [wslPath, ...filtered].slice(0, 5);
        });
        await invoke("update_wsl_directory", { directory: wslPath });
      }
    } catch (error) {
      console.error("Failed to browse WSL directory:", error);
    } finally {
      // Restore alwaysOnTop
      await currentWindow.setAlwaysOnTop(true);
      isDialogOpenRef.current = false;
      inputRef.current?.focus();
    }
  };

  const displayDirectory = currentDirectory ?? "(No directory selected)";
  const activeDirectories = isWsl ? wslRecentDirectories : recentDirectories;
  const activeDirectory = isWsl ? wslDirectory : currentDirectory;

  return (
    <div className="overlay-container" ref={containerRef}>
      <form ref={formRef} onSubmit={handleSubmit} className="input-form" onBlur={handleBlur}>
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
          {isWsl ? (
            <>
              <div className="wsl-directory-input-wrapper">
                <span className="directory-icon">&#128193;</span>
                <input
                  type="text"
                  value={wslDirectory}
                  onChange={(e) => setWslDirectory(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="~ or /home/user/project"
                  className="wsl-directory-input"
                />
                <button type="button" className="wsl-browse-button" onClick={handleWslBrowse}>
                  Browse
                </button>
                <button
                  type="button"
                  className="wsl-dropdown-toggle"
                  onClick={handleDirectoryClick}
                >
                  <span className="dropdown-arrow">{dropdownOpen ? "\u25B2" : "\u25BC"}</span>
                </button>
              </div>
              {dropdownOpen && (
                <div className="directory-dropdown">
                  {wslRecentDirectories.map((dir) => (
                    <button
                      type="button"
                      key={dir}
                      className={`dropdown-item ${dir === wslDirectory ? "active" : ""}`}
                      onClick={() => handleSelectDirectory(dir)}
                    >
                      {dir === wslDirectory && <span className="check-mark">&#9679;</span>}
                      <span className="dropdown-item-path">{dir}</span>
                    </button>
                  ))}
                  {wslRecentDirectories.length > 0 && <div className="dropdown-divider" />}
                  <button
                    type="button"
                    className="dropdown-item browse-item"
                    onClick={handleWslBrowse}
                  >
                    <span className="browse-icon">&#128194;</span>
                    <span>Browse...</span>
                  </button>
                </div>
              )}
            </>
          ) : (
            <>
              <button type="button" className="directory-button" onClick={handleDirectoryClick}>
                <span className="directory-icon">&#128193;</span>
                <span className="directory-path">{displayDirectory}</span>
                <span className="dropdown-arrow">{dropdownOpen ? "\u25B2" : "\u25BC"}</span>
              </button>
              {dropdownOpen && (
                <div className="directory-dropdown">
                  {activeDirectories.map((dir) => (
                    <button
                      type="button"
                      key={dir}
                      className={`dropdown-item ${dir === activeDirectory ? "active" : ""}`}
                      onClick={() => handleSelectDirectory(dir)}
                    >
                      {dir === activeDirectory && <span className="check-mark">&#9679;</span>}
                      <span className="dropdown-item-path">{dir}</span>
                    </button>
                  ))}
                  {activeDirectories.length > 0 && <div className="dropdown-divider" />}
                  <button
                    type="button"
                    className="dropdown-item browse-item"
                    onClick={handleBrowse}
                  >
                    <span className="browse-icon">&#128194;</span>
                    <span>Browse...</span>
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      </form>
    </div>
  );
}

export default App;
