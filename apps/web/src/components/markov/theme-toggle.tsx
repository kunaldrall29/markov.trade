import { useEffect, useState } from "react";
import { Moon, Sun } from "lucide-react";

function isDark() {
  return document.documentElement.classList.contains("dark");
}

export function ThemeToggle() {
  const [dark, setDark] = useState(false);

  useEffect(() => {
    setDark(isDark());
  }, []);

  function toggle() {
    const next = document.documentElement.classList.toggle("dark");
    try {
      localStorage.setItem("markov-theme", next ? "dark" : "light");
    } catch {
      /* ignore */
    }
    setDark(next);
  }

  return (
    <button
      type="button"
      onClick={toggle}
      className="grid size-11 place-items-center rounded-sm text-fg hover:bg-raised"
      aria-label={dark ? "Switch to light" : "Switch to dark"}
    >
      {dark ? <Sun className="size-4" strokeWidth={1.75} /> : <Moon className="size-4" strokeWidth={1.75} />}
    </button>
  );
}
