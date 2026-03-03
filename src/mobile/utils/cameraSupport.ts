export interface CameraSupportResult {
  supported: boolean;
  reason?:
    | "MEDIA_DEVICES_UNAVAILABLE"
    | "INSECURE_CONTEXT"
    | "PERMISSION_DENIED"
    | "NO_CAMERA"
    | "INIT_FAILED";
  detail?: string;
}

function isLocalhostHost(hostname: string) {
  return hostname === "localhost" || hostname === "127.0.0.1" || hostname === "::1";
}

export async function checkCameraSupport(): Promise<CameraSupportResult> {
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    return { supported: false, reason: "MEDIA_DEVICES_UNAVAILABLE" };
  }

  const secureContext = Boolean(window.isSecureContext);
  const localhost = isLocalhostHost(window.location.hostname);
  if (!secureContext && !localhost) {
    return { supported: false, reason: "INSECURE_CONTEXT" };
  }

  try {
    const stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: "environment" },
    });
    stream.getTracks().forEach((track) => track.stop());
    return { supported: true };
  } catch (err) {
    const e = err as { name?: string; message?: string };
    const name = typeof e?.name === "string" ? e.name : "";
    const message = typeof e?.message === "string" ? e.message : String(err);

    if (name === "NotAllowedError" || name === "PermissionDeniedError") {
      return { supported: false, reason: "PERMISSION_DENIED" };
    }
    if (name === "NotFoundError" || name === "DevicesNotFoundError") {
      return { supported: false, reason: "NO_CAMERA" };
    }

    return { supported: false, reason: "INIT_FAILED", detail: message };
  }
}
