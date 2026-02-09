import {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
} from "react";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
};

type ThemeProviderState = {
  theme: Theme;
  resolvedTheme: "light" | "dark";
  setTheme: (theme: Theme) => void | Promise<void>;
  toggleTheme: () => void;
  isLoading: boolean;
};

const initialState: ThemeProviderState = {
  theme: "system",
  resolvedTheme: "light",
  setTheme: () => {},
  toggleTheme: () => {},
  isLoading: true,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme(theme: "light" | "dark") {
  const root = document.documentElement;
  root.classList.remove("light", "dark");
  root.classList.add(theme);
}

export function ThemeProvider({
  children,
  defaultTheme = "system",
  ...props
}: ThemeProviderProps) {
  const [theme, setThemeState] = useState<Theme>(defaultTheme);
  const [resolvedTheme, setResolvedTheme] = useState<"light" | "dark">("light");
  const [isLoading, setIsLoading] = useState(true);

  const resolveTheme = useCallback((t: Theme): "light" | "dark" => {
    return t === "system" ? getSystemTheme() : t;
  }, []);

  const setTheme = useCallback(
    async (next: Theme) => {
      setThemeState(next);
      const resolved = resolveTheme(next);
      setResolvedTheme(resolved);
      applyTheme(resolved);

      try {
        await invoke("set_theme", { theme: next });
      } catch {}
    },
    [resolveTheme],
  );

  const toggleTheme = useCallback(() => {
    const order: Theme[] = ["system", "light", "dark"];
    const currentIndex = order.indexOf(theme);
    const next = order[(currentIndex + 1) % 3];
    setTheme(next);
  }, [theme, setTheme]);

  useEffect(() => {
    let cancelled = false;

    const init = async () => {
      try {
        const saved = await invoke<string>("get_theme");
        if (cancelled) return;

        let initialTheme: Theme;
        if (saved === "light" || saved === "dark" || saved === "system") {
          initialTheme = saved;
        } else {
          initialTheme = "system";
        }

        setThemeState(initialTheme);
        const resolved = resolveTheme(initialTheme);
        setResolvedTheme(resolved);
        applyTheme(resolved);
      } catch {
        if (cancelled) return;
        setThemeState("system");
        const resolved = getSystemTheme();
        setResolvedTheme(resolved);
        applyTheme(resolved);
      } finally {
        if (!cancelled) setIsLoading(false);
      }
    };

    init();

    return () => {
      cancelled = true;
    };
  }, [resolveTheme]);

  useEffect(() => {
    if (theme !== "system") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (e: MediaQueryListEvent) => {
      const systemTheme: "light" | "dark" = e.matches ? "dark" : "light";
      setResolvedTheme(systemTheme);
      applyTheme(systemTheme);
    };

    mediaQuery.addEventListener("change", onChange);
    return () => {
      mediaQuery.removeEventListener("change", onChange);
    };
  }, [theme]);

  const value = {
    theme,
    resolvedTheme,
    setTheme,
    toggleTheme,
    isLoading,
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider");

  return context;
};
