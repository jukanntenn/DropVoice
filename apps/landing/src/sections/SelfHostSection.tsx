import { useState } from 'react';
import { ArrowRight, Check, Copy } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button, Card } from '@dropvoice/ui';

import { SectionKicker } from '../components/SectionKicker';
import { SITE } from '../lib/site';

/**
 * 自托管（设计定稿）：mono 代码卡展示仓库里真实存在的 compose 片段
 * （节选，非虚构"一行跑起来"），复制按钮 + 指向 docker/ 目录的完整步骤链接。
 */
const COMPOSE_SNIPPET = `# 1) copy config from the repo: docker/config.example.toml
services:
  dropvoice-pairing:
    image: docker.io/jukanntenn/dropvoice-pairing-server:latest
    ports: ["8080:8080"]
    volumes:
      - ./config.toml:/app/config.toml:ro
      - dropvoice-data:/app/data
    restart: unless-stopped
# 2) docker compose up -d`;

export function SelfHostSection() {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(COMPOSE_SNIPPET);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      // 剪贴板不可用（非安全上下文等）时静默忽略。
    }
  };

  return (
    <section id="self-host">
      <div className="mx-auto grid max-w-6xl items-center gap-12 px-4 py-20 md:py-28 lg:grid-cols-2">
        <div>
          <SectionKicker>{t('landing:selfhost.kicker')}</SectionKicker>
          <h2 className="mt-3 text-2xl font-semibold tracking-tight text-neutral-900 md:text-3xl dark:text-white">
            {t('landing:selfhost.title')}
          </h2>
          <p className="mt-5 leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:selfhost.p1')}
          </p>
          <p className="mt-3 leading-relaxed text-neutral-600 dark:text-neutral-300">
            {t('landing:selfhost.p2')}
          </p>
          <a
            href={`${SITE.repo}/tree/main/apps/pairing-server/docker`}
            target="_blank"
            rel="noreferrer"
            className="text-primary-600 dark:text-primary-400 mt-5 inline-flex items-center gap-1 text-sm font-medium hover:underline"
          >
            {t('landing:selfhost.fullSteps')}
            <ArrowRight className="h-3.5 w-3.5" aria-hidden="true" />
          </a>
        </div>

        <Card className="overflow-hidden p-0">
          <div className="flex items-center justify-between border-b border-white/40 px-4 py-2.5 dark:border-white/10">
            <span className="font-mono text-xs text-neutral-500 dark:text-neutral-400">
              docker-compose.yml
            </span>
            <Button variant="ghost" size="sm" onClick={() => void copy()}>
              {copied ? (
                <Check className="h-3.5 w-3.5" aria-hidden="true" />
              ) : (
                <Copy className="h-3.5 w-3.5" aria-hidden="true" />
              )}
              {copied ? t('landing:selfhost.copyOk') : 'Copy'}
            </Button>
          </div>
          <pre className="overflow-x-auto p-4 font-mono text-xs leading-relaxed text-neutral-700 dark:text-neutral-200">
            {COMPOSE_SNIPPET}
          </pre>
        </Card>
      </div>
    </section>
  );
}
