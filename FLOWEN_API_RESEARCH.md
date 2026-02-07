# FLOWEN API RESEARCH - INNAN NÄSTA SESSION

## 🔍 INFORMATION VI BEHÖVER HITTA

### 1. FILE UPLOAD ENDPOINT

**Leta i dessa filer:**
```
src/app/api/files/upload/route.ts
src/app/api/storage/upload/route.ts
src/app/api/webdav/[...path]/route.ts
```

**Vad vi behöver veta:**
- [ ] Exakt URL path (t.ex. `/api/files/upload`)
- [ ] HTTP method (POST?)
- [ ] Request format:
```
  [ ] FormData med file field?
  [ ] JSON med base64 data?
  [ ] Multipart upload?
```
- [ ] Response format:
```json
  {
    "success": true,
    "fileId": "...",
    "url": "..."
  }
```

**Exempel från kod att leta efter:**
```typescript
export async function POST(request: Request) {
  const formData = await request.formData();
  const file = formData.get('file');
  // ...
}
```

---

### 2. AUTHENTICATION

**Bekräfta login endpoint:**
```
URL: POST /api/auth/login
Body: { email, password }
Response: { token }
```

**Headers för autentiserade requests:**
```
Authorization: Bearer {jwt_token}
```

---

### 3. KRYPTERING (SUPER VIKTIGT!)

**Leta i dessa filer:**
```
src/lib/encryption.ts
src/utils/encryption.ts
src/lib/crypto.ts
```

**Vad vi behöver:**
```typescript
// Exempel på vad vi letar efter:
import crypto from 'crypto';

const ALGORITHM = 'aes-256-gcm'; // Vilken algoritm?
const ENCRYPTION_KEY = process.env.ENCRYPTION_KEY; // Hur skapas key?

export function encryptFile(buffer: Buffer) {
  const iv = crypto.randomBytes(16); // Hur många bytes för IV?
  const cipher = crypto.createCipheriv(ALGORITHM, ENCRYPTION_KEY, iv);
  
  const encrypted = Buffer.concat([
    cipher.update(buffer),
    cipher.final()
  ]);
  
  const authTag = cipher.getAuthTag(); // GCM auth tag
  
  // Hur kombineras dessa? iv + authTag + encrypted?
  return {
    iv: iv.toString('hex'),
    encrypted: encrypted.toString('base64'),
    authTag: authTag.toString('hex')
  };
}
```

**Checklist:**
- [ ] Algoritm namn (aes-256-gcm, aes-256-cbc?)
- [ ] IV/Nonce längd (12 bytes? 16 bytes?)
- [ ] Key source (environment variable? Fixed key? Per-user key?)
- [ ] Key längd (32 bytes för AES-256)
- [ ] Auth tag längd (16 bytes för GCM?)
- [ ] Output format (hur kombineras IV + ciphertext + tag?)

---

### 4. FILE METADATA

**Vad skickas med filen?**
```typescript
// Leta efter POST body format:
{
  file: Binary,
  fileName: string,
  fileSize: number,
  mimeType: string,
  folderId?: string,
  companyId?: string,
  // Annat?
}
```

---

### 5. ENVIRONMENT VARIABLES

**Kolla .env.local eller .env:**
```bash
ENCRYPTION_KEY=...          # Vad är denna?
NEXT_PUBLIC_API_URL=...     # Base URL?
DATABASE_URL=...
```

**Vilka keys behöver Tauri-appen?**
- [ ] ENCRYPTION_KEY (för file encryption)
- [ ] API_BASE_URL (om inte hardcodad)
- [ ] Andra?

---

## 📝 TEMPLATE ATT FYLLA I

Kopiera denna template och fyll i info:
```yaml
# FLOWEN API CONFIG

## File Upload
endpoint: "/api/files/upload"
method: "POST"
content_type: "multipart/form-data"
body_format:
  - file: Binary
  - fileName: string
  - companyId: string

## Encryption
algorithm: "aes-256-gcm"
key_source: "ENCRYPTION_KEY env variable"
key_length: 32  # bytes
iv_length: 12   # bytes
auth_tag_length: 16  # bytes
output_format: "iv (12 bytes) + ciphertext + auth_tag (16 bytes)"

## Auth
login_endpoint: "/api/auth/login"
token_header: "Authorization: Bearer {token}"
token_expiry: 3600  # seconds

## Response Format
success:
  status: 200
  body:
    success: true
    fileId: "uuid"
    url: "https://..."

error:
  status: 400/401/500
  body:
    error: "message"
```

---

## 🧪 TEST I POSTMAN/CURL

### Test 1: Login
```bash
curl -X POST https://flowen.eu/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"daniel@industrinat.se","password":"..."}'
```

Förväntat svar:
```json
{
  "token": "eyJhbGc..."
}
```

### Test 2: Upload (med token från Test 1)
```bash
curl -X POST https://flowen.eu/api/files/upload \
  -H "Authorization: Bearer {TOKEN}" \
  -F "file=@test.txt" \
  -F "fileName=test.txt"
```

Förväntat svar:
```json
{
  "success": true,
  "fileId": "..."
}
```

---

## ✅ NÄR KLAR

Fyll i all info längst ner i denna fil under **"MINA FYND"** rubriken.

---

## 🚀 NÄSTA SESSION START

Med denna info kan vi:
1. Kompilera projektet (5 min)
2. Implementera file watcher (30 min)
3. Implementera login (20 min)
4. Implementera encryption (30 min)
5. Implementera upload (30 min)
6. Testa hela flödet (30 min)

**Total: ~2.5 timmar till fungerande MVP!**

---

## 📋 MINA FYND

*(Fyll i här när du hittat informationen)*

### File Upload Endpoint
```
URL: 
Method: 
Body format: 
Response format: 
```

### Encryption Details
```
Algorithm: 
Key source: 
Key length: 
IV length: 
Auth tag: 
Output format: 
```

### Testing Results
```
Login test: 
Upload test: 
```