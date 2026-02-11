"use client";

/**
 * WebAuthn passkey operations with PRF extension support.
 *
 * PRF (Pseudo-Random Function) returns deterministic 32-byte key material
 * during biometric auth, used to seed an Ed25519 keypair client-side.
 */

/** Check if the browser supports WebAuthn PRF extension. */
export async function isPrfSupported(): Promise<boolean> {
  if (typeof window === "undefined") return false;
  if (!window.PublicKeyCredential) return false;

  try {
    const available =
      await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
    if (!available) return false;

    // Use getClientCapabilities() if available (Chrome 131+, modern browsers).
    // This is the only reliable way to check PRF support without creating a credential.
    if ("getClientCapabilities" in PublicKeyCredential) {
      try {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const caps = await (PublicKeyCredential as any).getClientCapabilities();
        if (caps && typeof caps === "object" && "extension:prf" in caps) {
          return caps["extension:prf"] === true;
        }
      } catch {
        // getClientCapabilities threw — fall through to heuristic
      }
    }

    // Heuristic: platform authenticator exists but we can't verify PRF.
    // Return true optimistically — Chrome 116+ has PRF, but Brave doesn't.
    // The creation flow handles the case where PRF returns no output
    // by throwing PRF_UNSUPPORTED, which the onboarding UI catches
    // and falls back to password-based creation.
    return true;
  } catch {
    return false;
  }
}

/** Convert ArrayBuffer to base64url string. */
export function bufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let str = "";
  for (const b of bytes) str += String.fromCharCode(b);
  return btoa(str).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/** Convert base64url string to ArrayBuffer. */
export function base64urlToBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const pad = base64.length % 4;
  const padded = pad ? base64 + "=".repeat(4 - pad) : base64;
  const str = atob(padded);
  const bytes = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i);
  return bytes.buffer as ArrayBuffer;
}

interface CreatePasskeyResult {
  credentialId: string; // base64url
  prfOutput: Uint8Array | null; // 32 bytes if PRF supported
}

/**
 * Create a new passkey with PRF extension.
 * Returns the credential ID and PRF-derived key material.
 */
export async function createPasskeyWithPrf(
  rpId: string,
  userName: string,
  salt: Uint8Array
): Promise<CreatePasskeyResult> {
  const userId = new Uint8Array(32);
  crypto.getRandomValues(userId);

  const createOptions: CredentialCreationOptions = {
    publicKey: {
      rp: {
        id: rpId,
        name: "Norn Wallet",
      },
      user: {
        id: userId,
        name: userName,
        displayName: userName,
      },
      challenge: crypto.getRandomValues(new Uint8Array(32)),
      pubKeyCredParams: [
        { type: "public-key", alg: -7 }, // ES256 (P-256)
        { type: "public-key", alg: -257 }, // RS256
      ],
      authenticatorSelection: {
        authenticatorAttachment: "platform",
        residentKey: "required",
        userVerification: "required",
      },
      timeout: 60000,
      extensions: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        prf: {
          eval: {
            first: salt.buffer as ArrayBuffer,
          },
        },
      } as AuthenticationExtensionsClientInputs,
    },
  };

  const credential = (await navigator.credentials.create(
    createOptions
  )) as PublicKeyCredential;

  const credentialId = bufferToBase64url(credential.rawId);

  // Extract PRF output if available
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const prfResults = (credential.getClientExtensionResults() as any)?.prf;
  let prfOutput: Uint8Array | null = null;

  if (prfResults?.results?.first) {
    prfOutput = new Uint8Array(prfResults.results.first as ArrayBuffer);
    // Ensure exactly 32 bytes
    if (prfOutput.length !== 32) {
      // Hash to get exactly 32 bytes
      const { blake3Hash } = await import("@norn-protocol/sdk");
      prfOutput = blake3Hash(prfOutput);
    }
  }

  return { credentialId, prfOutput };
}

/**
 * Authenticate with an existing passkey and get PRF output.
 * Returns 32 bytes of deterministic key material.
 */
/**
 * Discover an existing passkey (no credential ID needed) and get PRF output.
 * Uses discoverable credentials — the OS shows a picker for available passkeys.
 * Returns the credential ID and 32 bytes of deterministic key material.
 */
export async function discoverPasskeyWithPrf(
  rpId: string,
  salt: Uint8Array
): Promise<{ credentialId: string; prfOutput: Uint8Array }> {
  const getOptions: CredentialRequestOptions = {
    publicKey: {
      challenge: crypto.getRandomValues(new Uint8Array(32)),
      rpId,
      userVerification: "required",
      timeout: 60000,
      extensions: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        prf: {
          eval: {
            first: salt.buffer as ArrayBuffer,
          },
        },
      } as AuthenticationExtensionsClientInputs,
    },
  };

  const assertion = (await navigator.credentials.get(
    getOptions
  )) as PublicKeyCredential;

  const credentialId = bufferToBase64url(assertion.rawId);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const prfResults = (assertion.getClientExtensionResults() as any)?.prf;

  if (!prfResults?.results?.first) {
    throw new Error("PRF extension not available or failed");
  }

  const raw = new Uint8Array(prfResults.results.first as ArrayBuffer);
  if (raw.length !== 32) {
    const { blake3Hash } = await import("@norn-protocol/sdk");
    return { credentialId, prfOutput: blake3Hash(raw) };
  }

  return { credentialId, prfOutput: raw };
}

export async function authenticateWithPrf(
  rpId: string,
  credentialId: string,
  salt: Uint8Array
): Promise<Uint8Array> {
  const allowCredentials: PublicKeyCredentialDescriptor[] = [
    {
      type: "public-key",
      id: base64urlToBuffer(credentialId),
    },
  ];

  const getOptions: CredentialRequestOptions = {
    publicKey: {
      challenge: crypto.getRandomValues(new Uint8Array(32)),
      rpId,
      allowCredentials,
      userVerification: "required",
      timeout: 60000,
      extensions: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        prf: {
          eval: {
            first: salt.buffer as ArrayBuffer,
          },
        },
      } as AuthenticationExtensionsClientInputs,
    },
  };

  const assertion = (await navigator.credentials.get(
    getOptions
  )) as PublicKeyCredential;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const prfResults = (assertion.getClientExtensionResults() as any)?.prf;

  if (!prfResults?.results?.first) {
    throw new Error("PRF extension not available or failed");
  }

  const raw = new Uint8Array(prfResults.results.first as ArrayBuffer);
  if (raw.length !== 32) {
    const { blake3Hash } = await import("@norn-protocol/sdk");
    return blake3Hash(raw);
  }

  return raw;
}
