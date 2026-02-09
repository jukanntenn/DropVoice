import { useCallback, useEffect, useRef, useState } from "react";

export type ConnectionStatus = "connected" | "disconnected" | "connecting";

interface UseWebSocketReturn {
  status: ConnectionStatus;
  isSending: boolean;
  lastError: string | null;
  send: (text: string) => boolean;
  reconnectAttempts: number;
}

const MAX_RECONNECT_ATTEMPTS = 5;

export function useWebSocket(): UseWebSocketReturn {
  const [status, setStatus] = useState<ConnectionStatus>("connecting");
  const [isSending, setIsSending] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const sendTimeoutRef = useRef<number | null>(null);
  const wasConnectedRef = useRef(false);

  const clearReconnectTimer = () => {
    if (reconnectTimeoutRef.current !== null) {
      window.clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  };

  const clearSendTimer = () => {
    if (sendTimeoutRef.current !== null) {
      window.clearTimeout(sendTimeoutRef.current);
      sendTimeoutRef.current = null;
    }
  };

  const scheduleReconnect = useCallback(() => {
    if (reconnectAttemptsRef.current >= MAX_RECONNECT_ATTEMPTS) return;

    reconnectAttemptsRef.current += 1;
    setReconnectAttempts(reconnectAttemptsRef.current);
    setStatus("connecting");

    const delay = Math.min(1000 * 2 ** (reconnectAttemptsRef.current - 1), 10000);
    clearReconnectTimer();
    reconnectTimeoutRef.current = window.setTimeout(() => {
      connect();
    }, delay);
  }, []);

  const connect = useCallback(() => {
    clearReconnectTimer();
    clearSendTimer();

    try {
      wsRef.current?.close();
    } catch {}

    wasConnectedRef.current = false;

    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    setStatus("connecting");

    try {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        wasConnectedRef.current = true;
        reconnectAttemptsRef.current = 0;
        setReconnectAttempts(0);
        setStatus("connected");
        setLastError(null);
        setIsSending(false);
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data?.type === "confirm") {
            clearSendTimer();
            setIsSending(false);
          } else if (data?.type === "error") {
            clearSendTimer();
            setLastError(typeof data.message === "string" ? data.message : "Error");
            setIsSending(false);
          }
        } catch {}
      };

      ws.onerror = () => {
        // Don't set error here - onerror fires during connection attempts
        // even when connection will succeed. Let onclose handle failures.
      };

      ws.onclose = () => {
        if (wsRef.current !== ws) return;
        wsRef.current = null;
        clearSendTimer();
        setIsSending(false);
        setStatus("disconnected");
        // Only show error if we never connected and this isn't the initial attempt
        if (!wasConnectedRef.current && reconnectAttemptsRef.current > 0) {
          setLastError("Connection error");
        }
        scheduleReconnect();
      };
    } catch {
      setStatus("disconnected");
      setLastError("Failed to create WebSocket");
      scheduleReconnect();
    }
  }, [scheduleReconnect]);

  useEffect(() => {
    connect();

    return () => {
      clearReconnectTimer();
      clearSendTimer();
      try {
        wsRef.current?.close();
      } catch {}
      wsRef.current = null;
    };
  }, [connect]);

  const send = useCallback(
    (text: string): boolean => {
      const ws = wsRef.current;
      if (!ws || ws.readyState !== WebSocket.OPEN) return false;
      if (isSending) return false;

      setIsSending(true);
      setLastError(null);

      try {
        ws.send(JSON.stringify({ type: "text", content: text }));
        clearSendTimer();
        sendTimeoutRef.current = window.setTimeout(() => {
          setIsSending(false);
        }, 3000);
        return true;
      } catch {
        clearSendTimer();
        setLastError("Failed to send");
        setIsSending(false);
        return false;
      }
    },
    [isSending],
  );

  return { status, isSending, lastError, send, reconnectAttempts };
}
