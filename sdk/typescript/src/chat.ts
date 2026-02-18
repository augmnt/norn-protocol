import { blake3Hash, blake3Kdf, ed25519Sign, ed25519Verify, publicKeyFromPrivate, toHex, fromHex } from "./crypto.js";
import { deriveSharedSecret, symmetricEncrypt, symmetricDecrypt } from "./encryption.js";
import type { ChatEvent } from "./types.js";

/**
 * Create and sign a chat event (Nostr-inspired, Ed25519 + BLAKE3).
 *
 * The event ID is the BLAKE3 hash of [pubkey, created_at, kind, tags_json, content].
 * The signature is Ed25519 over the raw ID bytes.
 */
export function createChatEvent(
  privateKey: Uint8Array,
  kind: number,
  content: string,
  tags: string[][],
): ChatEvent {
  const pubkey = toHex(publicKeyFromPrivate(privateKey));
  const created_at = Math.floor(Date.now() / 1000);

  // Compute event ID: BLAKE3 hash of serialized fields
  const encoder = new TextEncoder();
  const tagsJson = JSON.stringify(tags);
  const preimage = new Uint8Array([
    ...encoder.encode(pubkey),
    ...encoder.encode(String(created_at)),
    ...encoder.encode(String(kind)),
    ...encoder.encode(tagsJson),
    ...encoder.encode(content),
  ]);
  const idBytes = blake3Hash(preimage);
  const id = toHex(idBytes);

  // Sign the raw ID bytes with Ed25519
  const sig = toHex(ed25519Sign(idBytes, privateKey));

  return { id, pubkey, created_at, kind, tags, content, sig };
}

/**
 * Verify a chat event's ID and signature.
 * Returns true if the ID matches the recomputed hash and the signature is valid.
 */
export function verifyChatEvent(event: ChatEvent): boolean {
  // Recompute event ID
  const encoder = new TextEncoder();
  const tagsJson = JSON.stringify(event.tags);
  const preimage = new Uint8Array([
    ...encoder.encode(event.pubkey),
    ...encoder.encode(String(event.created_at)),
    ...encoder.encode(String(event.kind)),
    ...encoder.encode(tagsJson),
    ...encoder.encode(event.content),
  ]);
  const expectedId = toHex(blake3Hash(preimage));

  if (expectedId !== event.id) return false;

  // Verify Ed25519 signature over ID bytes
  try {
    const idBytes = fromHex(event.id);
    const sigBytes = fromHex(event.sig);
    const pubkeyBytes = fromHex(event.pubkey);
    return ed25519Verify(sigBytes, idBytes, pubkeyBytes);
  } catch {
    return false;
  }
}

/**
 * Encrypt DM content for a recipient.
 * Uses deterministic shared secret derived from both parties' keys.
 *
 * Returns the base64-encoded ciphertext as content and nonce tag for inclusion in the event.
 */
export function encryptDmContent(
  myPrivateKey: Uint8Array,
  recipientX25519Public: Uint8Array,
  plaintext: string,
): { content: string; nonceTags: string[][] } {
  const sharedKey = deriveSharedSecret(myPrivateKey, recipientX25519Public);
  const encoder = new TextEncoder();
  const { nonce, ciphertext } = symmetricEncrypt(sharedKey, encoder.encode(plaintext));

  // Encode ciphertext as base64, nonce as hex
  const content = btoa(String.fromCharCode(...ciphertext));
  const nonceTags: string[][] = [["nonce", toHex(nonce)]];

  return { content, nonceTags };
}

/**
 * Decrypt DM content from a sender.
 * Uses the same deterministic shared secret.
 */
export function decryptDmContent(
  myPrivateKey: Uint8Array,
  senderX25519Public: Uint8Array,
  content: string,
  nonceHex: string,
): string {
  const sharedKey = deriveSharedSecret(myPrivateKey, senderX25519Public);
  const ciphertext = Uint8Array.from(atob(content), (c) => c.charCodeAt(0));
  const nonce = fromHex(nonceHex);
  const decoder = new TextDecoder();
  return decoder.decode(symmetricDecrypt(sharedKey, nonce, ciphertext));
}
