Utmärkt! 🚀
STEG 8: Skapa IMPLEMENTATION_GUIDE.md
powershellcode IMPLEMENTATION_GUIDE.md
Kör detta kommando ↑
Klistra in detta (LÅNG FIL - ta din tid):
markdown# FLOWEN SYNC - IMPLEMENTATION GUIDE
## Nästa Session: Implementera File Watcher + Upload

---

## ✅ KLART (denna session)
- [x] Alla dependencies tillagda i Cargo.toml
- [x] Stub-funktioner skapade för:
  - start_watching()
  - stop_watching()
  - login_to_flowen()
  - upload_file_to_flowen()
  - encrypt_file_data()
- [x] State management (JWT token, watching status)
- [x] Frontend kan anropa alla commands

---

## 🔨 ATT IMPLEMENTERA - STEG FÖR STEG

### **STEG 1: FILE WATCHER (30-45 min)**

#### Fil: `main.rs` → Funktion `start_watching()`
```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::mpsc::channel;

#[tauri::command]
async fn start_watching(folder_path: String, state: State) -> Result {
    let (tx, rx) = channel();
    
    let mut watcher = notify::recommended_watcher(move |res: Result| {
        if let Ok(event) = res {
            tx.send(event).unwrap();
        }
    }).map_err(|e| format!("Watcher error: {}", e))?;
    
    watcher.watch(
        std::path::Path::new(&folder_path), 
        RecursiveMode::Recursive
    ).map_err(|e| format!("Watch error: {}", e))?;
    
    // Loop för att hantera events
    loop {
        if let Ok(event) = rx.recv() {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in event.paths {
                        println!("📁 File changed: {:?}", path);
                        // TODO: Anropa upload_file_to_flowen här
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok("Watching started".to_string())
}
```

**Testplan:**
1. Kör `npm run tauri dev`
2. Klicka "Start Syncing"
3. Skapa en fil på E:\test.txt
4. Kolla konsolen → ska logga "File changed"

---

### **STEG 2: FLOWEN LOGIN (30 min)**

#### API Endpoint Information

**URL:** `POST https://flowen.eu/api/auth/login`

**Request:**
```json
{
  "email": "daniel@industrinat.se",
  "password": "ditt_lösenord"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

#### Implementering i `main.rs` → Funktion `login_to_flowen()`
```rust
use reqwest;

#[tauri::command]
async fn login_to_flowen(email: String, password: String, state: State) -> Result {
    let client = reqwest::Client::new();
    
    let login_data = serde_json::json!({
        "email": email,
        "password": password
    });
    
    let response = client
        .post("https://flowen.eu/api/auth/login")
        .json(&login_data)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Login failed: {}", response.status()));
    }
    
    let login_response: LoginResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;
    
    // Spara token
    let mut token = state.jwt_token.lock().unwrap();
    *token = Some(login_response.token.clone());
    
    Ok("Logged in successfully".to_string())
}
```

**Testplan:**
1. Anropa `login_to_flowen` från frontend
2. Verifiera att JWT token sparas
3. Kolla att token är giltig (kan göra test-request)

---

### **STEG 3: FILE UPLOAD (45 min)**

#### VIKTIGT: Behöver info från befintlig Flowen-kod

**Frågor att besvara:**
1. Vilken API endpoint används för file upload i Flowen?
   - Troligen: `POST /api/files/upload` eller `/api/storage/upload`
   
2. Hur fungerar krypteringen exakt?
   - Kolla i Flowen Next.js kod: Leta efter AES kryptering
   - Vilken key används?
   - Hur genereras IV/nonce?

3. Vilket format förväntar sig API:et?
   - FormData med fil?
   - JSON med base64?
   - Multipart upload?

#### Implementering (exempel med FormData)
```rust
use reqwest::multipart;

#[tauri::command]
async fn upload_file_to_flowen(file_path: String, state: State) -> Result {
    // 1. Kontrollera token
    let token = state.jwt_token.lock().unwrap();
    let token_str = token.as_ref().ok_or("Not logged in")?;
    
    // 2. Läs fil
    let file_data = read_file(&file_path).await?;
    
    // 3. Kryptera (TODO: implementera encrypt_file_data)
    let encrypted_data = encrypt_file_data(file_data).await?;
    
    // 4. Skapa multipart form
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?;
    
    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(encrypted_data)
            .file_name(file_name.to_string()));
    
    // 5. POST till Flowen
    let client = reqwest::Client::new();
    let response = client
        .post("https://flowen.eu/api/files/upload") // VERIFIERA URL!
        .header("Authorization", format!("Bearer {}", token_str))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Upload failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Upload failed: {}", response.status()));
    }
    
    Ok(format!("Uploaded: {}", file_path))
}
```

---

### **STEG 4: KRYPTERING (45 min)**

**KRITISKT:** Krypteringen måste matcha exakt vad Flowen redan använder!

#### Leta i Flowen Next.js kod efter:
```typescript
// Leta efter något liknande:
import crypto from 'crypto';

const algorithm = 'aes-256-gcm';
const key = process.env.ENCRYPTION_KEY; // Hur genereras denna?

function encrypt(buffer: Buffer) {
  const iv = crypto.randomBytes(16);
  const cipher = crypto.createCipheriv(algorithm, key, iv);
  // ...
}
```

#### När du hittat krypteringslogiken i Flowen, implementera samma i Rust:
```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::Rng;

async fn encrypt_file_data(data: Vec) -> Result<Vec, String> {
    // VIKTIGT: Key måste vara samma som i Flowen!
    // Hur hämtar vi key? Environment variable? Config fil?
    let key_bytes = [0u8; 32]; // TODO: Hämta riktig key!
    
    let cipher = Aes256Gcm::new(&key_bytes.into());
    
    let mut rng = rand::thread_rng();
    let nonce_bytes: [u8; 12] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Returnera: nonce + ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}
```

---

## 📋 CHECKLIST INNAN VI BÖRJAR KODA

### Från Flowen Next.js projekt, hitta:
- [ ] File upload API endpoint URL
- [ ] Request format (FormData/JSON/Base64?)
- [ ] Krypteringsalgoritm detaljer:
  - [ ] Exact algorithm (AES-256-GCM?)
  - [ ] Key source (env variable? config?)
  - [ ] IV/Nonce generation
  - [ ] Output format
- [ ] Response format från API
- [ ] Eventuella extra headers som behövs

### Filer att kolla i Flowen:
```
src/app/api/files/upload/route.ts  (eller liknande)
src/lib/encryption.ts               (eller liknande)
.env.local                          (för ENCRYPTION_KEY)
```

---

## 🧪 TESTPLAN

### Test 1: Compilation
```bash
cd C:\projects\flowen-desktop-sync
cargo build
```
→ Ska kompilera utan errors

### Test 2: File Watcher
1. `npm run tauri dev`
2. Start syncing
3. Skapa fil på E:\
4. Se att den loggas i konsol

### Test 3: Login
1. Anropa login från UI
2. Verifiera att token sparas
3. Logga token till konsol

### Test 4: Upload
1. Login
2. Skapa liten testfil (test.txt, 1KB)
3. Upload
4. Verifiera i Flowen web att den finns

### Test 5: Industrinat Data
1. Allt ovan fungerar
2. Kopiera Industrinat folders → E:\
3. Låt sync köra
4. Verifiera i Flowen

---

## 🚨 KÄNDA PROBLEM & LÖSNINGAR

### Problem: File watcher kan trigga flera gånger för samma fil
**Lösning:** Debounce events (vänta 500ms innan upload)

### Problem: Stora filer tar lång tid att uploada
**Lösning:** Chunked upload (implementera senare)

### Problem: Kryptering är långsam
**Lösning:** Använd tokio för async, kryptera i bakgrund

### Problem: Token expires
**Lösning:** Spara refresh token, auto-refresh

---

## 📊 TID PER STEG (estimat)

- File Watcher: 30-45 min
- Login: 30 min  
- Upload: 45 min
- Kryptering: 45 min
- Testing: 30 min
- **Total: ~3 timmar**

---

## 🎯 FRAMTIDA FEATURES (efter MVP)

1. Progress bar för uploads
2. Queue för fler filer samtidigt
3. Retry logic vid misslyckade uploads
4. Konflikhantering (samma fil ändrad på två ställen)
5. Selective sync (välj vilka mappar)
6. Bandwidth throttling
7. Offline queue (spara till upload när internet tillbaka)

---

**KLART! Du är redo för nästa session! 🚀**