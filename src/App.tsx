import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { initI18n } from "./i18n";
import Header from "./components/Header";
import LanWarning from "./components/LanWarning";
import QRCodeSection from "./components/QRCodeSection";
import URLDisplay from "./components/URLDisplay";
import { TooltipProvider } from "./components/ui/tooltip";
import { ThemeProvider } from "./components/ThemeProvider";

interface ConnectionInfo {
  isRunning: boolean;
  url: string;
  connectionCount: number;
}

function App() {
  const { t, i18n } = useTranslation();
  const [i18nInitialized, setI18nInitialized] = useState(false);
  const [serverUrl, setServerUrl] = useState<string>("");

  const [error, setError] = useState<string>("");

  useEffect(() => {
    initI18n().then(() => setI18nInitialized(true));
  }, []);

  useEffect(() => {
    const lang = i18n.language?.startsWith("zh") ? "zh" : "en";
    document.documentElement.lang = lang;
  }, [i18n.language]);

  useEffect(() => {
    const startServer = async () => {
      try {
        const url = await invoke<string>("start_server");
        setServerUrl(url);
      } catch (err) {
        const message = typeof err === "string" ? err : "";
        if (message && message !== "Server is already running") {
          setError(message);
        }
      }
    };

    startServer();

    const interval = setInterval(async () => {
      try {
        const info = await invoke<ConnectionInfo>("get_connection_info");
        if (info.url) setServerUrl(info.url);
      } catch (err) {}
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  if (!i18nInitialized) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-teal-50 via-white to-cyan-50 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground dark:text-slate-400">
            Loading...
          </p>
        </div>
      </div>
    );
  }

  return (
    <ThemeProvider>
      <TooltipProvider>
        <div className="flex min-h-screen flex-col bg-gradient-to-br from-teal-50 via-white to-cyan-50 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
          {/* Decorative background elements */}
          <div className="pointer-events-none fixed inset-0 overflow-hidden">
            <div className="absolute -top-40 -right-40 h-80 w-80 rounded-full bg-primary/10 blur-3xl" />
            <div className="absolute -bottom-40 -left-40 h-80 w-80 rounded-full bg-cyan-400/10 blur-3xl dark:bg-cyan-500/5" />
          </div>

          <Header />

          <main className="relative flex flex-1 flex-col items-center justify-start gap-8 md:px-12">
            {error ? (
              <div className="animate-slide-up w-full max-w-sm rounded-2xl border border-red-200/80 bg-red-50/80 px-6 py-4 text-sm text-red-700 backdrop-blur-sm dark:border-red-500/20 dark:bg-red-950/30 dark:text-red-300">
                {error}
              </div>
            ) : null}

            <LanWarning />

            {serverUrl ? (
              <div className="animate-slide-up flex w-full max-w-sm flex-col items-center gap-8">
                <div className="w-full mt-8">
                  <QRCodeSection url={serverUrl} />
                </div>

                <URLDisplay url={serverUrl} />
              </div>
            ) : (
              <div className="flex flex-col items-center gap-3">
                <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                <p className="text-sm text-muted-foreground dark:text-slate-400">
                  {t("app.loading")}
                </p>
              </div>
            )}
          </main>
        </div>
      </TooltipProvider>
    </ThemeProvider>
  );
}

export default App;
