import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { QUERY_KEYS } from '@dropvoice/core';

import { tauriInvoke } from '../lib/invoke';

/**
 * 轮询服务器连接信息（每 1 秒），驱动 active_connections/clients/queue_depth 等 UI。
 *
 * §6：配对码由 `pairing_code_rotated` 事件总线即时更新（App.tsx 订阅）；本 1s
 * 轮询保留作为 active_connections/clients/queue_depth 的主源（无专用事件），并对
 * 配对码提供兜底（get_connection_info 不再触发轮换）。
 */
export function useServerState() {
  return useQuery({
    queryKey: QUERY_KEYS.connectionInfo,
    queryFn: tauriInvoke.getConnectionInfo,
    refetchInterval: 1000,
  });
}

/**
 * 启动 HTTP/WebSocket 服务器。成功后刷新连接信息缓存。
 */
export function useStartServer() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: tauriInvoke.startServer,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.connectionInfo });
    },
  });
}

/**
 * 停止服务器。成功后刷新连接信息缓存。
 */
export function useStopServer() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: tauriInvoke.stopServer,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.connectionInfo });
    },
  });
}
