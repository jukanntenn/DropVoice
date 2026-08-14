import { test, expect, type Page } from '@playwright/test';

/**
 * E2E：设备管理 UI（§8/§9 黑盒）。
 *
 * WebRTC 架构下端到端连接需真实 pairing-server + 桌面端，e2e 环境不具备；
 * 但设备列表（持久身份）+ 单一持久化 + 发送模式选择是纯前端可黑盒覆盖的：
 * - 注入 v2 设备数据（与 §9 瘦身后的形状一致），验证水合后只显示身份字段、
 *   瞬态总是干净（无陈旧 status）。
 * - 删活跃设备 -> 刷新 -> 不复活（§8 验收）。
 * - 切换活跃设备 -> 刷新 -> 列表恢复（§8 验收）。
 *
 * 这些用例对状态正确性做断言，正是规范验收 #3 的要求。
 *
 * UI 契约（apps/mobile）：
 * - 设备 chip 是 radiogroup[aria-label="Select device"] 内的 radio（label=device.name）。
 * - 删除经 "Devices"（MoreVertical，aria-label="Devices"）菜单，每设备一个
 *   menuitem[aria-label="Remove Device"]。
 * - 发送模式选择器是另一组 radiogroup[aria-label="Select target device"]（>=2 设备才出现）。
 */

const DEVICE_STORAGE_KEY = 'dropvoice:devices:v2';
const TEST_DEVICE = { id: 'test-pc', name: 'Test PC', autoConnect: false };

/** 注入 v2 设备列表 + lastActiveDeviceId。
 *
 * 注意：`page.addInitScript` 在**每次**文档导航（含 reload）时都会重放，
 * 会用原始 seed 覆盖被编辑过的 localStorage。对需要"刷新后断言变更存活"的
 * 用例，用 `once` + 持久 marker（localStorage key，跨 reload 存活）确保仅
 * 首次注入，后续导航不再覆盖（否则删除/切换的持久化结果会被原始 seed 抹掉，
 * 造成 §8 实现正确却误报失败）。
 */
async function seedDevices(
  page: Page,
  devices: Array<{ id: string; name: string; autoConnect: boolean }>,
  lastActiveDeviceId: string | null,
  once = false
): Promise<void> {
  await page.addInitScript(
    ({ key, devices, lastActiveDeviceId, once }) => {
      const marker = '__test_seeded';
      if (once && window.localStorage.getItem(marker)) return;
      window.localStorage.setItem(marker, '1');
      window.localStorage.setItem(key, JSON.stringify({ devices, lastActiveDeviceId }));
    },
    { key: DEVICE_STORAGE_KEY, devices, lastActiveDeviceId, once }
  );
}

test.describe('Device management (§8/§9)', () => {
  test('hydrates device list and shows empty state when none', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByText('No devices connected')).toBeVisible();
  });

  test('hydrates v2 devices (identity only) and shows the active device name', async ({ page }) => {
    await seedDevices(page, [TEST_DEVICE], 'test-pc');
    await page.goto('/');
    // 设备 chip label = device.name（§9 身份字段）。
    await expect(page.getByRole('radio', { name: 'Test PC' })).toBeVisible();
  });

  test('stale transient fields from legacy data are stripped on hydrate (§9/§0.4)', async ({
    page,
  }) => {
    // 模拟 v2 键下残留的旧瞬态字段；水合后 UI 只应认身份。
    await page.addInitScript(
      ({ key, device, lastActiveDeviceId }) => {
        window.localStorage.setItem(
          key,
          JSON.stringify({
            devices: [{ ...device, status: 'connected', lastConnected: 999, errorType: 'timeout' }],
            lastActiveDeviceId,
          })
        );
      },
      { key: DEVICE_STORAGE_KEY, device: TEST_DEVICE, lastActiveDeviceId: 'test-pc' }
    );
    await page.goto('/');
    // 设备仍可见（身份保留）；但连接瞬态不被信任（UI 不会显示已连接）。
    await expect(page.getByRole('radio', { name: 'Test PC' })).toBeVisible();
    // 未连接（瞬态被丢弃）-> 不显示 "Connected" 状态丸文案。
    await expect(page.getByText('Connected', { exact: true })).toHaveCount(0);
  });

  test('removing the active device survives refresh (§8 single-effect persistence)', async ({
    page,
  }) => {
    const devices = [
      { id: 'pc-a', name: 'PC-A', autoConnect: false },
      { id: 'pc-b', name: 'PC-B', autoConnect: false },
    ];
    await seedDevices(page, devices, 'pc-a', true);
    await page.goto('/');

    // 打开 "Devices" 管理菜单，移除活跃设备 pc-a（首个 Remove Device menuitem）。
    await page.getByRole('button', { name: 'Devices' }).click();
    await page.getByRole('menuitem', { name: 'Remove Device' }).first().click();

    // §8 单一持久化 effect 在 paint 后 flush 到 localStorage；直接轮询 storage
    // 确认 pc-a 已被写回移除（避免 reload 抢在 effect flush 之前的竞态）。
    await expect
      .poll(
        async () => {
          const raw = await page.evaluate(
            (key) => window.localStorage.getItem(key),
            DEVICE_STORAGE_KEY
          );
          const parsed = raw ? (JSON.parse(raw) as { devices: { id: string }[] }) : null;
          return parsed?.devices.some((d) => d.id === 'pc-a') ?? true;
        },
        { timeout: 5_000 }
      )
      .toBe(false);

    // 刷新 -> pc-a 不复活（§8 验收：删任意设备刷新不复活）。
    await page.reload();
    await expect(page.getByRole('radio', { name: 'PC-A' })).toHaveCount(0);
    await expect(page.getByRole('radio', { name: 'PC-B' })).toBeVisible();
  });

  test('switching active device persists across refresh (§8 验收)', async ({ page }) => {
    const devices = [
      { id: 'pc-a', name: 'PC-A', autoConnect: false },
      { id: 'pc-b', name: 'PC-B', autoConnect: false },
    ];
    await seedDevices(page, devices, 'pc-a', true);
    await page.goto('/');

    // 切换活跃设备到 pc-b（点击其 chip radio -> onSelect）。
    await page.getByRole('radio', { name: 'PC-B' }).click();

    // 刷新 -> 列表恢复（pc-a、pc-b 都在）。
    await page.reload();
    await expect(page.getByRole('radio', { name: 'PC-A' })).toBeVisible();
    await expect(page.getByRole('radio', { name: 'PC-B' })).toBeVisible();
  });

  test('send-mode selector only appears with >=2 devices (spec 10 §2.4)', async ({ page }) => {
    await seedDevices(page, [TEST_DEVICE], 'test-pc');
    await page.goto('/');
    // 单设备时仅 1 个 radiogroup（设备选择器）；发送模式选择器（第 2 个）不渲染。
    await expect(page.getByRole('radiogroup')).toHaveCount(1);

    await seedDevices(
      page,
      [
        { id: 'pc-a', name: 'PC-A', autoConnect: false },
        { id: 'pc-b', name: 'PC-B', autoConnect: false },
      ],
      'pc-a'
    );
    await page.goto('/');
    // 双设备时出现发送模式选择器（设备选择器 1 + 发送模式选择器 1 = 2 个 radiogroup）。
    await expect(page.getByRole('radiogroup')).toHaveCount(2);
    // 发送模式选择器含 3 个 radio（active/all/selected）。
    const allRadios = page.getByRole('radio');
    // 2 设备 chip + 3 发送模式 radio = 5。
    await expect(allRadios).toHaveCount(5);
  });
});
