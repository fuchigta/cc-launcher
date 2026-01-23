import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
  const [prompt, setPrompt] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const currentWindow = getCurrentWindow();

    // Focus input when window becomes visible
    const unlistenFocus = currentWindow.onFocusChanged(({ payload: focused }) => {
      if (focused && inputRef.current) {
        inputRef.current.focus();
        inputRef.current.select();
      }
    });

    // Initial focus
    if (inputRef.current) {
      inputRef.current.focus();
    }

    return () => {
      unlistenFocus.then((unlisten) => unlisten());
    };
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (prompt.trim()) {
      try {
        await invoke("open_claude_interactive", { prompt: prompt.trim() });
        setPrompt("");
        await invoke("hide_window");
      } catch (error) {
        console.error("Failed to launch Claude:", error);
      }
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setPrompt("");
      await invoke("hide_window");
    }
  };

  const handleBlur = async () => {
    // Small delay to allow click events to complete
    setTimeout(async () => {
      const currentWindow = getCurrentWindow();
      const isFocused = await currentWindow.isFocused();
      if (!isFocused) {
        setPrompt("");
        await invoke("hide_window");
      }
    }, 100);
  };

  return (
    <div className="overlay-container">
      <form onSubmit={handleSubmit} className="input-form">
        <input
          ref={inputRef}
          type="text"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          placeholder="Ask Claude..."
          className="prompt-input"
          autoFocus
        />
      </form>
    </div>
  );
}

export default App;
