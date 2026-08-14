# Clones open source libraries used by DropVoice into .local/contexts/
# so their source/docs can be inspected to verify correct usage.

$ErrorActionPreference = 'Continue'
$dest = 'c:\Users\Administrator\Workspace\dropvoice\.local\contexts'

# Format: repo_url, target_dir
$repos = @(
  @{ url = 'https://github.com/mui/base-ui.git'; dir = 'base-ui' },
  @{ url = 'https://github.com/tokio-rs/axum.git'; dir = 'axum' },
  @{ url = 'https://github.com/enigo-rs/enigo.git'; dir = 'enigo' },
  @{ url = 'https://github.com/TanStack/query.git'; dir = 'tanstack-query' },
  @{ url = 'https://github.com/jotaijs/jotai.git'; dir = 'jotai' },
  @{ url = 'https://github.com/emilkowalski/sonner.git'; dir = 'sonner' },
  @{ url = 'https://github.com/zpao/qrcode.react.git'; dir = 'qrcode.react' },
  @{ url = 'https://github.com/mebjas/html5-qrcode.git'; dir = 'html5-qrcode' },
  @{ url = 'https://github.com/lucide-icons/lucide.git'; dir = 'lucide' },
  @{ url = 'https://github.com/motiondivision/motion.git'; dir = 'motion' },
  @{ url = 'https://github.com/colinhacks/zod.git'; dir = 'zod' },
  @{ url = 'https://github.com/react-hook-form/react-hook-form.git'; dir = 'react-hook-form' },
  @{ url = 'https://github.com/i18next/i18next.git'; dir = 'i18next' },
  @{ url = 'https://github.com/i18next/react-i18next.git'; dir = 'react-i18next' },
  @{ url = 'https://github.com/tokio-rs/tracing.git'; dir = 'tracing' },
  @{ url = 'https://github.com/open-telemetry/opentelemetry-rust.git'; dir = 'opentelemetry-rust' },
  @{ url = 'https://github.com/joe-bell/cva.git'; dir = 'cva' },
  @{ url = 'https://github.com/dcastil/tailwind-merge.git'; dir = 'tailwind-merge' },
  @{ url = 'https://github.com/lukeed/clsx.git'; dir = 'clsx' },
  @{ url = 'https://github.com/tauri-apps/tauri.git'; dir = 'tauri' },
  @{ url = 'https://github.com/dtolnay/thiserror.git'; dir = 'thiserror' },
  @{ url = 'https://github.com/snapview/tokio-tungstenite.git'; dir = 'tokio-tungstenite' },
  @{ url = 'https://github.com/EstebanBorai/local-ip-address.git'; dir = 'local-ip-address' },
  @{ url = 'https://github.com/tauri-apps/plugins-workspace.git'; dir = 'tauri-plugins' },
  @{ url = 'https://github.com/i18next/i18next-browser-languagedetector.git'; dir = 'i18next-browser-languagedetector' }
)

function Clone-Repo {
  param($repo)
  $target = Join-Path $dest $repo.dir
  if (Test-Path $target) {
    Write-Host "[skip] $($repo.dir) already exists"
    return
  }
  Write-Host "[clone] $($repo.dir) <- $($repo.url)"
  & git clone --depth 1 $repo.url $target 2>&1 | Out-Null
  if (-not (Test-Path $target)) {
    Write-Host "[fail]  $($repo.dir)"
  } else {
    Write-Host "[ok]    $($repo.dir)"
  }
}

# Clone sequentially to avoid rate limits and network saturation
foreach ($repo in $repos) {
  Clone-Repo $repo
}

Write-Host "Done cloning contexts."
