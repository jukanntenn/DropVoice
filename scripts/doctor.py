#!/usr/bin/env python3
"""DropVoice 开发/部署前置条件体检（pnpm doctor）。

逐项检查工具链、质量门、dev/accept/deploy 三条路径的前置条件，输出
[ok] / [warn] / [fail]。任何 [fail] 退出码 1；[warn] 不影响退出码
（缺该项只影响对应路径，如 deploy 需要 vault/ssh 而 dev 不需要）。

纯标准库、跨平台（Windows / macOS / Linux）。
"""

from __future__ import annotations

import shutil
import socket
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# dev 三件套 + 容器验收占用的端口（被占只 warn——很可能就是正在跑的服务）。
DEV_PORTS = {
    7380: "pairing-server（dev 裸跑，tasks.json env 注入）",
    5173: "desktop Vite dev server",
    5174: "mobile PWA dev server",
    8080: "容器验收 Caddy（pnpm accept:up）",
}

failures = 0


def report(level: str, message: str, hint: str = "") -> None:
    global failures
    line = f"[{level}] {message}"
    # 提示只对非 ok 项有意义（ok 项打印修复提示是噪音）。
    if level != "ok" and hint:
        line += f"\n       提示: {hint}"
    print(line)
    if level == "fail":
        failures += 1


def command_version(cmd: str, args: list[str]) -> str | None:
    """Run `cmd args` and return stdout, or None if unavailable.

    Windows 上 CreateProcess 对裸名不做 PATHEXT 解析（pnpm 是 pnpm.CMD），
    因此必须先经 shutil.which 解析成完整路径再执行。
    """
    resolved = shutil.which(cmd)
    if resolved is None:
        return None
    try:
        out = subprocess.run(
            [resolved, *args], capture_output=True, text=True, timeout=30
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    return out.stdout.strip() if out.returncode == 0 else None


def check_toolchain() -> None:
    print("== 工具链 ==")
    node = command_version("node", ["--version"])
    if node is None:
        report("fail", "node 未安装（要求 >=22）", "https://nodejs.org 或 nvm")
    else:
        major = int(node.lstrip("v").split(".")[0])
        report(
            "ok" if major >= 22 else "fail",
            f"node {node}（要求 >=22）",
        )

    pnpm = command_version("pnpm", ["--version"])
    if pnpm is None:
        report("fail", "pnpm 未安装（要求 >=11）", "corepack enable 或 npm i -g pnpm")
    else:
        major = int(pnpm.split(".")[0])
        report("ok" if major >= 11 else "fail", f"pnpm {pnpm}（要求 >=11）")

    cargo = command_version("cargo", ["--version"])
    report(
        "ok" if cargo else "fail",
        f"cargo {cargo.split()[1]}" if cargo else "cargo 未安装（desktop / pairing-server 必需）",
        "https://rustup.rs",
    )


def check_quality_gate() -> None:
    print("== 质量门 ==")
    pre_commit = REPO_ROOT / ".git" / "hooks" / "pre-commit"
    if pre_commit.exists():
        report("ok", "prek git hooks 已安装")
    else:
        report("fail", "prek git hooks 未安装", "prek install（每个 clone 一次）")


def check_dev_prereqs() -> None:
    print("== dev（dev:full 三件套，Alt+R）==")
    # dev 配置全部由 .vscode/tasks.json 的 env 注入（自包含）：
    # LISTEN_ADDR / DATABASE_URL / RATE_LIMIT_PER_SEC / PAIRING_SERVER_URL /
    # VITE_API_PROXY_TARGET。config.local.toml 不再是 dev 前置条件——env
    # 优先级高于 TOML（见 apps/pairing-server/src/config.rs），文件仅作
    # 裸跑（不经任务）时的可选便利。
    tasks = REPO_ROOT / ".vscode" / "tasks.json"
    report(
        "ok" if tasks.exists() else "fail",
        ".vscode/tasks.json 存在（dev 配置唯一来源）"
        if tasks.exists()
        else ".vscode/tasks.json 缺失 —— dev:full 无法自包含启动",
    )
    dev_db = REPO_ROOT / "apps" / "pairing-server" / "dev.db"
    if dev_db.exists():
        report(
            "ok",
            "apps/pairing-server/dev.db 存在（dev 数据库，已 gitignore）",
        )
    else:
        report(
            "ok",
            "apps/pairing-server/dev.db 尚未创建（首次 dev:full 自动创建）",
        )


def check_ports() -> None:
    print("== 端口占用（被占通常意味着服务已在跑，不算错误）==")
    for port, what in DEV_PORTS.items():
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.bind(("127.0.0.1", port))
            report("ok", f":{port} 空闲（{what}）")
        except OSError:
            report("warn", f":{port} 被占用（{what}）")


def check_deploy_prereqs() -> None:
    print("== deploy（pnpm deploy:staging / production）==")
    if shutil.which("docker") is None:
        report("warn", "docker 未安装 —— 镜像构建与 ansible runner 容器都依赖它", "Docker Desktop")
        return
    info = subprocess.run(
        ["docker", "info"], capture_output=True, text=True, timeout=30
    )
    report(
        "ok" if info.returncode == 0 else "warn",
        "docker daemon 可用" if info.returncode == 0 else "docker 已装但 daemon 未运行",
        "启动 Docker Desktop / dockerd（仅 deploy:*/accept:*/image:push 需要）",
    )

    home = Path.home()
    for env in ("staging", "production"):
        pwd_file = home / ".ansible-vault" / f"dropvoice-{env}.pwd"
        exists = pwd_file.exists()
        state = "存在" if exists else "不存在"
        report(
            "ok" if exists else "warn",
            f"vault 密码文件 ~/.ansible-vault/dropvoice-{env}.pwd {state}",
            f"仅 deploy:{env} 需要",
        )
    report(
        "ok" if (home / ".ssh").exists() else "warn",
        "~/.ssh 存在" if (home / ".ssh").exists() else "~/.ssh 不存在",
        "deploy:* 经 ssh 连接目标主机",
    )


def main() -> int:
    print("DropVoice doctor —— 前置条件体检\n")
    check_toolchain()
    check_quality_gate()
    check_dev_prereqs()
    check_ports()
    check_deploy_prereqs()
    print()
    if failures:
        print(f"结果: {failures} 项 [fail]，请先修复再继续。")
        return 1
    print("结果: 无 [fail]；[warn] 只影响对应路径（见各条提示）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
