// src/crypto.ts
export async function encryptFile(fileData: ArrayBuffer): Promise<{
  encryptedData: Uint8Array;
  key: string;
  iv: string;
  authTag: string;
}> {
  // 1. Generera random 32-byte key
  const key = crypto.getRandomValues(new Uint8Array(32));
  
  // 2. Generera random 12-byte IV
  const iv = crypto.getRandomValues(new Uint8Array(12));
  
  // 3. Importera key för AES-GCM
  const cryptoKey = await crypto.subtle.importKey(
    'raw',
    key,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt']
  );
  
  // 4. Kryptera
  const encrypted = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: iv,
      tagLength: 128  // 16 bytes authTag
    },
    cryptoKey,
    fileData
  );
  
  // 5. encrypted innehåller: [ciphertext + authTag (sista 16 bytes)]
  const encryptedArray = new Uint8Array(encrypted);
  const ciphertext = encryptedArray.slice(0, -16);
  const authTag = encryptedArray.slice(-16);
  
  // 6. Bygg resultat: IV + authTag + ciphertext (för kompatibilitet)
  const result = new Uint8Array(12 + 16 + ciphertext.length);
  result.set(iv, 0);
  result.set(authTag, 12);
  result.set(ciphertext, 28);
  
  // 7. Base64 encode nycklar
  const keyBase64 = btoa(String.fromCharCode(...key));
  const ivBase64 = btoa(String.fromCharCode(...iv));
  const authTagBase64 = btoa(String.fromCharCode(...authTag));
  
  return {
    encryptedData: result,
    key: keyBase64,
    iv: ivBase64,
    authTag: authTagBase64
  };
}