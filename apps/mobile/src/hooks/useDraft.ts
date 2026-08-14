import { useCallback, useEffect, useState } from 'react';

import { getDraft, setDraft as persistDraft } from '../lib/storage';

/**
 * 草稿持久化 hook：按 deviceId 自动加载 / 保存草稿到 localStorage。
 *
 * Key: `dropvoice:draft:{deviceId}`（规范 10 section 2.6）。
 */
export function useDraft(deviceId: string | null): [string, (text: string) => void] {
  const [draft, setDraftState] = useState<string>('');

  // 切换设备时加载草稿。
  useEffect(() => {
    if (!deviceId) {
      setDraftState('');
      return;
    }
    setDraftState(getDraft(deviceId));
  }, [deviceId]);

  const setDraft = useCallback(
    (text: string) => {
      setDraftState(text);
      if (deviceId) {
        persistDraft(deviceId, text);
      }
    },
    [deviceId]
  );

  return [draft, setDraft];
}
