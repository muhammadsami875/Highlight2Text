import { Component, type ReactNode, type ErrorInfo } from "react";
import { AlertTriangle, RotateCcw } from "lucide-react";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, info.componentStack);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex items-center justify-center min-h-[400px] p-8">
          <div className="text-center max-w-md">
            <AlertTriangle
              size={48}
              className="mx-auto mb-4 text-amber-500"
            />
            <h2 className="text-lg font-bold text-surface-900 dark:text-surface-100 mb-2">
              Something went wrong
            </h2>
            <p className="text-sm text-surface-500 dark:text-surface-400 mb-4">
              An unexpected error occurred. You can try again or restart the
              application.
            </p>
            {this.state.error && (
              <details className="mb-4 text-left">
                <summary className="text-xs text-surface-400 cursor-pointer">
                  Technical details
                </summary>
                <pre className="mt-2 p-3 rounded-lg bg-surface-100 dark:bg-surface-800 text-xs text-surface-600 dark:text-surface-400 whitespace-pre-wrap font-mono overflow-auto max-h-32">
                  {this.state.error.message}
                </pre>
              </details>
            )}
            <button onClick={this.handleReset} className="btn-primary">
              <RotateCcw size={16} className="mr-2 inline" />
              Try Again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
