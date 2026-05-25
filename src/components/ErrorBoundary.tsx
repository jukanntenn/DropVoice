import { Component, ErrorInfo, ReactNode } from "react";
import i18n from "../i18n";

interface Props {
  children?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error:", error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen w-screen flex-col items-center justify-center gap-4 bg-background p-4 text-foreground dark:text-gray-200">
          <h1 className="text-2xl font-bold text-destructive dark:text-red-400">
            {i18n.t("app.error")}
          </h1>
          <pre className="max-w-full overflow-auto rounded bg-muted p-4 text-xs dark:text-gray-300">
            {this.state.error?.toString()}
          </pre>
        </div>
      );
    }

    return this.props.children;
  }
}
