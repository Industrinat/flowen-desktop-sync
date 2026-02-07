3. **Research Flowen API** (15 min)
   - Öppna Flowen Next.js projektet
   - Leta efter file upload endpoint
   - Hitta krypteringslogik
   - Fyll i FLOWEN_API_RESEARCH.md

### Under kodning:

4. **Implementera File Watcher** (30-45 min)
   - Använd notify crate
   - Logga alla file events
   - Testa med små filer

5. **Implementera Login** (30 min)
   - POST till /api/auth/login
   - Spara JWT token
   - Testa med dina credentials

6. **Implementera Upload** (45 min)
   - Läs fil från disk
   - Kryptera (matcha Flowen's encryption)
   - POST till API
   - Testa med 1KB testfil

7. **End-to-End Test** (30 min)
   - Login → Upload → Verifiera i Flowen web
   - Test med större fil (10MB)
   - Test med Industrinat data om allt fungerar

---

## 🎯 MÅLET NÄSTA SESSION

**MVP som kan:**
1. ✅ Bevaka E:\ för nya filer
2. ✅ Logga in mot Flowen
3. ✅ Ladda upp filer med kryptering
4. ✅ Verifiera i Flowen web att filer finns

**När MVP fungerar:**
→ Börja synka Industrinat 2022, 2023, 2024 data

---

## 📂 FILER SKAPADE DENNA SESSION