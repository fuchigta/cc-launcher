import { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { TerminalType } from "./types";
import {
  getConfig,
  updateWslDirectory,
  openClaudeInteractive,
  updateRecentDirectory,
  hideWindow,
  getWslRootPath,
  uncToWslPath,
} from "./commands";

function App() {
  const [prompt, setPrompt] = useState("");
  const [terminal, setTerminal] = useState<TerminalType>("Auto");
  const [currentDirectory, setCurrentDirectory] = useState<string | null>(null);
  const [recentDirectories, setRecentDirectories] = useState<string[]>([]);
  const [wslDirectory, setWslDirectory] = useState<string>("");
  const [wslRecentDirectories, setWslRecentDirectories] = useState<string[]>([]);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const isSubmittingRef = useRef(false);
  const formRef = useRef<HTMLFormElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const isDialogOpenRef = useRef(false);
  const compositionJustEndedRef = useRef(false);
  const cursorPosRef = useRef<number | null>(null);

  const isWsl = terminal === "Wsl";

  useEffect(() => {
    const loadConfig = async () => {
      const config = await getConfig();
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

    const unlistenDirectory = listen<string>("set-directory", (event) => {
      const dir = event.payload;
      setCurrentDirectory(dir);
      setRecentDirectories((prev) => {
        const filtered = prev.filter((d) => d !== dir);
        return [dir, ...filtered].slice(0, 5);
      });
      updateRecentDirectory(dir);
    });

    return () => {
      unlistenFocus.then((unlisten) => unlisten());
      unlistenDirectory.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    if (cursorPosRef.current !== null && inputRef.current) {
      inputRef.current.setSelectionRange(cursorPosRef.current, cursorPosRef.current);
      cursorPosRef.current = null;
    }
  }, [prompt]);

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

  // Resize window when dropdown opens/closes or textarea grows
  useEffect(() => {
    const currentWindow = getCurrentWindow();
    // 61 = non-textarea overhead (container padding + directory-row margin + directory-row height)
    const textareaHeight = inputRef.current?.offsetHeight ?? 59;
    const baseHeight = textareaHeight + 61;
    const bottomPadding = 12;
    // Calculate dropdown height: items (40px each) + browse (40px) + divider (9px) + margin (4px)
    const dirList = isWsl ? wslRecentDirectories : recentDirectories;
    const browseHeight = 40;
    const dropdownHeight = dirList.length * 40 + browseHeight + (dirList.length > 0 ? 9 : 0) + 4;
    const expandedHeight = baseHeight + dropdownHeight + bottomPadding;

    if (dropdownOpen) {
      currentWindow.setSize(new LogicalSize(600, expandedHeight));
    } else {
      currentWindow.setSize(new LogicalSize(600, baseHeight + bottomPadding));
    }
  }, [dropdownOpen, recentDirectories.length, wslRecentDirectories.length, isWsl, prompt]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (isSubmittingRef.current) return;
    if (prompt.trim()) {
      isSubmittingRef.current = true;
      try {
        setError(null);
        if (isWsl) {
          if (wslDirectory.trim()) {
            await updateWslDirectory(wslDirectory.trim());
          }
          await openClaudeInteractive(prompt.trim(), null);
        } else {
          if (currentDirectory) {
            await updateRecentDirectory(currentDirectory);
          }
          await openClaudeInteractive(prompt.trim(), currentDirectory);
        }
        setPrompt("");
        await hideWindow();
      } catch (err) {
        console.error("Failed to launch Claude:", err);
        setError(String(err));
      } finally {
        isSubmittingRef.current = false;
      }
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.nativeEvent.isComposing) {
      if (compositionJustEndedRef.current) {
        compositionJustEndedRef.current = false;
        return;
      }
      if (e.ctrlKey) {
        e.preventDefault();
        const target = e.target as HTMLTextAreaElement;
        const start = target.selectionStart ?? prompt.length;
        const end = target.selectionEnd ?? prompt.length;
        setPrompt(prompt.slice(0, start) + "\n" + prompt.slice(end));
        cursorPosRef.current = start + 1;
        return;
      }
      e.preventDefault();
      if (!isSubmittingRef.current) {
        formRef.current?.requestSubmit();
      }
    } else if (e.key === "Escape") {
      if (dropdownOpen) {
        setDropdownOpen(false);
      } else {
        setPrompt("");
        await hideWindow();
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
        await hideWindow();
      }
    }, 100);
  };

  const handleDirectoryClick = useCallback(() => {
    setDropdownOpen((prev) => !prev);
  }, []);

  const handleSelectDirectory = useCallback(
    (dir: string) => {
      if (isWsl) {
        setWslDirectory(dir);
      } else {
        setCurrentDirectory(dir);
      }
      setDropdownOpen(false);
      inputRef.current?.focus();
    },
    [isWsl],
  );

  const handleBrowse = async (forWsl: boolean) => {
    isDialogOpenRef.current = true;
    setDropdownOpen(false);

    const currentWindow = getCurrentWindow();

    try {
      await currentWindow.setAlwaysOnTop(false);

      const defaultPath = forWsl ? await getWslRootPath() : (currentDirectory ?? undefined);

      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath,
      });

      if (selected && typeof selected === "string") {
        if (forWsl) {
          const wslPath = await uncToWslPath(selected);
          setWslDirectory(wslPath);
          setWslRecentDirectories((prev) => {
            const filtered = prev.filter((d) => d !== wslPath);
            return [wslPath, ...filtered].slice(0, 5);
          });
          await updateWslDirectory(wslPath);
        } else {
          setCurrentDirectory(selected);
          setRecentDirectories((prev) => {
            const filtered = prev.filter((d) => d !== selected);
            return [selected, ...filtered].slice(0, 5);
          });
          await updateRecentDirectory(selected);
        }
      }
    } catch (err) {
      console.error("Failed to browse directory:", err);
      setError(String(err));
    } finally {
      await currentWindow.setAlwaysOnTop(true);
      isDialogOpenRef.current = false;
      inputRef.current?.focus();
    }
  };

  const displayDirectory = currentDirectory ?? "(No directory selected)";
  const activeDirectories = isWsl ? wslRecentDirectories : recentDirectories;
  const activeDirectory = isWsl ? wslDirectory : currentDirectory;

  const renderDropdown = (dirs: string[], activeDir: string | null, onBrowse: () => void) => (
    <div className="directory-dropdown">
      {dirs.map((dir) => (
        <button
          type="button"
          key={dir}
          className={`dropdown-item ${dir === activeDir ? "active" : ""}`}
          onClick={() => handleSelectDirectory(dir)}
        >
          {dir === activeDir && <span className="check-mark">&#9679;</span>}
          <span className="dropdown-item-path">{dir}</span>
        </button>
      ))}
      {dirs.length > 0 && <div className="dropdown-divider" />}
      <button type="button" className="dropdown-item browse-item" onClick={onBrowse}>
        <span className="browse-icon">&#128194;</span>
        <span>Browse...</span>
      </button>
    </div>
  );

  return (
    <div className="overlay-container" ref={containerRef}>
      <form ref={formRef} onSubmit={handleSubmit} className="input-form" onBlur={handleBlur}>
        <textarea
          ref={inputRef}
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onCompositionEnd={() => {
            compositionJustEndedRef.current = true;
          }}
          onKeyDown={handleKeyDown}
          placeholder="Ask Claude..."
          className="prompt-input"
          autoFocus
          rows={1}
        />
        {error && <div className="error-message">{error}</div>}
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
                <button
                  type="button"
                  className="wsl-browse-button"
                  onClick={() => handleBrowse(true)}
                >
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
              {dropdownOpen &&
                renderDropdown(wslRecentDirectories, wslDirectory, () => handleBrowse(true))}
            </>
          ) : (
            <>
              <button type="button" className="directory-button" onClick={handleDirectoryClick}>
                <span className="directory-icon">&#128193;</span>
                <span className="directory-path">{displayDirectory}</span>
                <span className="dropdown-arrow">{dropdownOpen ? "\u25B2" : "\u25BC"}</span>
              </button>
              {dropdownOpen &&
                renderDropdown(activeDirectories, activeDirectory, () => handleBrowse(false))}
            </>
          )}
        </div>
      </form>
    </div>
  );
}

export default App;
