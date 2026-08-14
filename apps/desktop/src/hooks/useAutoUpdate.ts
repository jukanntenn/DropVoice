/**
 * useAutoUpdate — checks for app updates on mount (spec 09 §4).
 *
 * Uses the Tauri updater plugin to check for updates. If an update is
 * available, shows a toast notification with an "Update" action button.
 * The update is downloaded and installed in the background.
 */
import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from '@dropvoice/ui';

export function useAutoUpdate() {
  const { t } = useTranslation();
  const checkAttemptedRef = useRef(false);

  useEffect(() => {
    if (checkAttemptedRef.current) return;
    checkAttemptedRef.current = true;

    // Dynamic import to avoid crash if updater plugin is not available
    // (e.g., in dev mode or when running from source).
    async function checkForUpdates() {
      try {
        const { check } = await import('@tauri-apps/plugin-updater');
        const update = await check();
        if (update) {
          toast.info(t('common:update.available', { defaultValue: 'Update available' }), {
            description: t('common:update.description', {
              defaultValue: `Version ${update.version} is available`,
              version: update.version,
            }),
            action: {
              label: t('common:update.install', { defaultValue: 'Update' }),
              onClick: async () => {
                try {
                  await update.downloadAndInstall();
                  toast.success(
                    t('common:update.installed', {
                      defaultValue: 'Update installed. Restart to apply.',
                    })
                  );
                } catch (err) {
                  console.error('Update install failed:', err);
                  toast.error(t('common:update.failed', { defaultValue: 'Update failed' }));
                }
              },
            },
            duration: 30000,
          });
        }
      } catch {
        // Updater plugin not available or check failed — silent.
      }
    }

    // Delay the check slightly so the app has time to start.
    const timer = setTimeout(checkForUpdates, 5000);
    return () => clearTimeout(timer);
  }, [t]);
}
