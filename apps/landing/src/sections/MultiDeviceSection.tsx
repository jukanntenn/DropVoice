import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { Device } from '@dropvoice/core/types';
import { Card, ConnectionStatus, DeviceSelector } from '@dropvoice/ui';

import { SectionKicker } from '../components/SectionKicker';

/**
 * 深潜 B · 多设备管理（设计定稿）：直接复用产品的 DeviceSelector 真组件
 * （确定性色点即设备身份），点击切换发送目标——深潜章节本身是可交互的。
 */
export function MultiDeviceSection() {
  const { t } = useTranslation();
  const devices: Device[] = [
    { id: 'workstation', name: t('landing:devices.d1'), autoConnect: true },
    { id: 'laptop', name: t('landing:devices.d2'), autoConnect: true },
    { id: 'home', name: t('landing:devices.d3'), autoConnect: false },
  ];
  const [activeId, setActiveId] = useState(devices[0]?.id ?? null);
  const active = devices.find((d) => d.id === activeId);

  return (
    <section id="devices">
      <div className="mx-auto grid max-w-6xl items-center gap-12 px-4 py-20 md:py-28 lg:grid-cols-2">
        <Card className="order-2 flex flex-col gap-4 p-6 md:p-8 lg:order-1">
          <DeviceSelector devices={devices} activeDeviceId={activeId} onSelect={setActiveId} />

          <div className="mt-2 flex items-center justify-between rounded-xl border border-white/60 bg-white/50 px-4 py-3 dark:border-white/10 dark:bg-slate-800/50">
            <div>
              <p className="text-xs text-neutral-400 dark:text-neutral-500">
                {t('landing:devices.switchHint')}
              </p>
              <p className="mt-0.5 text-lg font-semibold text-neutral-900 dark:text-white">
                {active?.name ?? '—'}
              </p>
            </div>
            <ConnectionStatus status="connected" latency={12} />
          </div>
        </Card>

        <div className="order-1 lg:order-2">
          <SectionKicker>{t('landing:devices.kicker')}</SectionKicker>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight text-neutral-900 md:text-3xl dark:text-white">
            {t('landing:devices.title')}
          </h2>
          <p className="mt-5 leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:devices.p1')}
          </p>
        </div>
      </div>
    </section>
  );
}
