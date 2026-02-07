# FLOWEN DESKTOP SYNC - DOKUMENTATION
## 📁 Automatisk fil-synkning mellan lokal disk och Flowen Cloud

---

## 🚀 SNABBSTART

### Du är här nu:
**✅ STEG 1 KLART** - Dependencies och stub-funktioner skapade

### Nästa steg:
**→ STEG 2** - Implementera funktionalitet

**LÄS I DENNA ORDNING:**

1. **INSTALLATION.md** ← BÖRJA HÄR
   - Hur du installerar uppdateringarna
   - Ersätt Cargo.toml och main.rs
   - Kör cargo build

2. **FLOWEN_API_RESEARCH.md** ← GÖR INNAN KODNING
   - Hitta API endpoints i Flowen
   - Dokumentera kryptering
   - Testa med curl/Postman

3. **IMPLEMENTATION_GUIDE.md** ← ANVÄND UNDER KODNING
   - Steg-för-steg implementation
   - Kod-exempel
   - Testplan

4. **SESSION_SUMMARY.md** ← ÖVERSIKT
   - Vad vi gjort hittills
   - Vad som återstår
   - Tidsestimat

---

## 📊 PROJEKT STATUS
```
COMPLETED:
✅ Tauri app setup
✅ React UI med settings
✅ E:\ drive mounting (subst)
✅ Folder creation
✅ Dependencies added (notify, reqwest, aes-gcm, tokio)
✅ Stub functions for all features
✅ State management (JWT token, watching status)

TO DO:
⏳ Implement file watcher with notify crate
⏳ Implement Flowen API login
⏳ Implement AES-256-GCM encryption
⏳ Implement file upload to Flowen
⏳ End-to-end testing
⏳ Sync Industrinat data (2022, 2023, 2024)
```

---

## 🏗️ ARKITEKTUR
```
┌─────────────────────┐
│   React Frontend    │
│   (Settings UI)     │
└──────────┬──────────┘
           │ Tauri Commands
┌──────────▼──────────┐
│   Rust Backend      │
│   (main.rs)         │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    │             │
┌───▼───┐   ┌────▼────┐
│  E:\  │   │ Flowen  │
│ (subst│   │   API   │
│ drive)│   │  HTTPS  │
└───┬───┘   └────┬────┘
    │            │
┌───▼────────────▼───┐
│  File Operations   │
│  Watch → Encrypt   │
│  → Upload          │
└────────────────────┘
```

---

## 🛠️ TEKNOLOGI STACK

### Frontend
- React + TypeScript
- Tauri invoke() för backend calls

### Backend (Rust)
- **notify** - File system watcher
- **reqwest** - HTTP client för API
- **tokio** - Async runtime
- **aes-gcm** - AES-256-GCM kryptering
- **serde** - JSON serialization

### Integration
- Flowen REST API
- JWT authentication
- Encrypted file storage

---

## 📋 NÄSTA SESSION CHECKLIST

**INNAN DU BÖRJAR KODA:**
- [ ] Läs INSTALLATION.md
- [ ] Ersätt Cargo.toml och main.rs
- [ ] Kör `cargo build` (tar 5-10 min första gången)
- [ ] Fyll i FLOWEN_API_RESEARCH.md
- [ ] Testa API med curl/Postman

**UNDER KODNING:**
- [ ] Implementera file watcher
- [ ] Implementera login
- [ ] Implementera kryptering
- [ ] Implementera upload
- [ ] Testa med liten fil (1KB)
- [ ] Testa med större fil (10MB)
- [ ] Verifiera i Flowen webb

**NÄR KLART:**
- [ ] Synka Industrinat 2022 data
- [ ] Synka Industrinat 2023 data
- [ ] Synka Industrinat 2024 data

---

## 📖 FILER I DETTA PAKET

| Fil | Syfte | Prioritet |
|-----|-------|-----------|
| **README.md** | Denna fil - översikt | ⭐ Läs först |
| **INSTALLATION.md** | Installationsguide | ⭐⭐⭐ Börja här |
| **FLOWEN_API_RESEARCH.md** | API dokumentation att fylla i | ⭐⭐⭐ Gör innan kodning |
| **IMPLEMENTATION_GUIDE.md** | Steg-för-steg kodguide | ⭐⭐⭐ Använd under kodning |
| **SESSION_SUMMARY.md** | Sammanfattning av vad vi gjort | ⭐ För översikt |

---

## ⏱️ TIDSESTIMAT

### Denna session (KLART):
- ✅ Analys och planering: 10 min
- ✅ Dependencies setup: 10 min
- ✅ Stub functions: 15 min
- ✅ Dokumentation: 15 min
- **Total: 50 minuter**

### Nästa session (TODO):
- Research API: 15 min
- Installation: 10 min
- File Watcher: 45 min
- Login: 30 min
- Encryption + Upload: 45 min
- Testing: 30 min
- **Total: ~2.5-3 timmar**

---

## 🎯 MÅLLINJE

**MVP Definition:**
- Bevaka E:\ för nya filer
- Automatisk upload till Flowen
- Krypterade filer
- Verifierat i Flowen webb

**Success Criteria:**
1. Skapa fil på E:\test.txt
2. Fil syns i Flowen webb inom 10 sekunder
3. Fil kan dekrypteras och laddas ner i Flowen
4. Industrinat data (10GB+) synkas framgångsrikt

---

## 💪 DU ÄR REDO!

Du har nu allt du behöver för att slutföra projektet:
- ✅ Komplett arkitektur
- ✅ Alla dependencies
- ✅ Stub-funktioner att fylla i
- ✅ Steg-för-steg guides
- ✅ Test-plan
- ✅ Troubleshooting tips

**Börja med INSTALLATION.md → Sen FLOWEN_API_RESEARCH.md → Sen IMPLEMENTATION_GUIDE.md**

**LYCKA TILL!** 🚀